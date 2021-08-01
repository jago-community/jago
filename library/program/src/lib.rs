use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use proc_macro::TokenStream;
use proc_macro2_diagnostics::{Diagnostic, Level};

use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Program {
    features: HashMap<String, Vec<String>>,
    dependencies: HashMap<String, Dependency>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Dependency {
    Version(String),
    Specification(DependencySpecification),
}

#[derive(Debug, Deserialize)]
struct DependencySpecification {
    path: String,
    #[serde(default)]
    optional: bool,
}

#[proc_macro]
pub fn build(_input: TokenStream) -> TokenStream {
    let configuration = include_str!("../../../Cargo.toml");

    let program: Program = match toml::from_str(configuration) {
        Ok(program) => program,
        Err(error) => {
            println!("{}", error);
            return Diagnostic::new(Level::Error, "Unable to parse Cargo.toml")
                .error(error.to_string())
                .emit_as_expr_tokens()
                .into();
        }
    };

    let mut before = quote!();
    let mut body = quote!();
    let mut error_kinds = quote!();
    let mut error_formatters = quote!();
    let mut error_sourcers = quote!();
    let mut error_transformers = quote!();

    let mut features = program
        .dependencies
        .iter()
        .filter(|(_name, dependency)| match dependency {
            Dependency::Specification(specification) => specification.optional,
            _ => false,
        })
        .map(|(package, _)| {
            let identity = format_ident!("{}", package);

            let library_path = PathBuf::from("library")
                .join(package)
                .join("src")
                .join("lib.rs");

            inspect_library(&library_path)
                .map(|(level, options)| (package, identity, level, options))
        })
        .collect::<Vec<_>>();

    features.sort_by(|a, b| match (a, b) {
        (Ok((_, _, a, _)), Ok((_, _, b, _))) => a.cmp(b),
        _ => std::cmp::Ordering::Less,
    });

    for feature in features {
        let (package, identity, _, options) = match feature {
            Ok(feature) => feature,
            Err(error) => return error.emit_as_expr_tokens().into(),
        };

        if options.contains(LibraryOptions::BEFORE) {
            before.extend(quote! {
                #[cfg(feature = #package)]
                match #identity::before() {
                    Ok(Some(cleanup)) => {
                        after.push(cleanup);
                    }
                    Ok(None) => {}
                    Err(error) => {
                        eprintln!("error running {}::before : {}", stringify!(#package), error);
                        code = 1;
                    }
                };
            });
        }

        if options.contains(LibraryOptions::HANDLE) {
            body.extend(quote! {
                #[cfg(feature = #package)]
                match #identity::handle(&mut input) {
                    Err(error) => match &error {
                        #identity::Error::Incomplete => {}
                        _ => {
                            eprintln!("error handling {} input: {}", stringify!(#package), error);
                            code = 1;
                        }
                    },
                    _ => {}
                };
            });
        }

        let formatted = utility::unicode::to_upper_camel_case(package);
        let formatted = format_ident!("{}", formatted);

        error_kinds.extend(quote! {
            #[cfg(feature = #package)]
            #formatted(#identity::Error),
        });

        error_formatters.extend(quote! {
            #[cfg(feature = #package)]
            Error::#formatted(error) => error.fmt(f),
        });

        error_sourcers.extend(quote! {
            #[cfg(feature = #package)]
            Error::#formatted(error) => Some(error),
        });

        error_transformers.extend(quote! {
            #[cfg(feature = #package)]
            impl From<#identity::Error> for Error {
                fn from(error: #identity::Error) -> Self {
                    Self::#formatted(error)
                }
            }
        });
    }

    TokenStream::from(quote! {
        fn main() {
            let start = std::time::Instant::now();

            let mut input = std::env::args().skip(1).peekable();
            let mut code = 0;
            let mut after: Vec<Box<dyn Fn()>> = vec![];

            #before

            log::info!("starting execution");

            #body

            log::info!("{:?} elapsed", start.elapsed());

            after.iter().for_each(|handle| handle());

            std::process::exit(code);
        }

        #[derive(Debug)]
        enum Error {
            #error_kinds
        }

        impl std::fmt::Display for Error {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #error_formatters
                    _ => unreachable!()
                }
            }
        }

        impl std::error::Error for Error {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                match self {
                    #error_sourcers
                    _ => unreachable!()
                }
            }
        }

        #error_transformers
    })
}

bitflags::bitflags! {
    struct LibraryOptions: u32 {
        const BEFORE = 0b00000001;
        const HANDLE = 0b00000010;
    }
}

fn inspect_library(library_path: &Path) -> Result<(usize, LibraryOptions), Diagnostic> {
    let mut file = File::open(library_path).map_err(|error| {
        Diagnostic::new(
            Level::Error,
            format!("Unable to open file {}", library_path.display()),
        )
        .error(error.to_string())
    })?;

    let mut source = String::new();

    file.read_to_string(&mut source).map_err(|error| {
        Diagnostic::new(Level::Error, "Unable to read file").error(error.to_string())
    })?;

    let tree = syn::parse_file(&source).map_err(|error| {
        Diagnostic::new(
            Level::Error,
            format!("Unable to open file {}", library_path.display()),
        )
        .error(error.to_string())
    })?;

    let mut pass = 2;
    let mut options = LibraryOptions::empty();

    for item in tree.items.iter() {
        match item {
            syn::Item::Fn(ref function) => match &function.sig.ident.to_string()[..] {
                "before" => options.insert(LibraryOptions::BEFORE),
                "handle" => options.insert(LibraryOptions::HANDLE),
                _ => {}
            },
            syn::Item::Static(ref static_item) => match &static_item.ident.to_string()[..] {
                "PASS" => match static_item.expr.as_ref() {
                    syn::Expr::Lit(ref expression) => match expression.lit {
                        syn::Lit::Int(ref literal) => match literal.base10_parse() {
                            Ok(literal) => {
                                pass = literal;
                            }
                            _ => {
                                return Err(Diagnostic::new(
                                    Level::Error,
                                    "Expected: `PASS: usize`",
                                ))
                            }
                        },
                        _ => return Err(Diagnostic::new(Level::Error, "Expected: `PASS: usize`")),
                    },
                    _ => return Err(Diagnostic::new(Level::Error, "Expected: `PASS: usize`")),
                },
                _ => {}
            },
            _ => {}
        };
    }

    Ok((pass, options))
}

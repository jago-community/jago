mod unicode;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use proc_macro::TokenStream;
use proc_macro2_diagnostics::{Diagnostic, Level};
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Platform {
    features: HashMap<String, Vec<String>>,
}

#[proc_macro]
pub fn build(_input: TokenStream) -> TokenStream {
    let configuration = include_str!("../../../Cargo.toml");

    let platform: Platform = match toml::from_str(configuration) {
        Ok(platform) => platform,
        Err(error) => {
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

    for (feature, crates) in &platform.features {
        if feature == "default" {
            continue;
        }

        for handler in crates {
            let handler_ident = format_ident!("{}", handler);

            let library_path = PathBuf::from("library")
                .join(handler)
                .join("src")
                .join("lib.rs");

            match has_before(&library_path) {
                Ok(true) => {
                    println!("{} has before", handler);

                    before.extend(quote! {
                        match #handler_ident::before() {
                            Ok(cleanup) => {
                                after = Some(cleanup);
                            }
                            Err(error) => {
                                eprintln!("error starting logger: {}", error);
                                code = 1;
                            }
                        };
                    });
                }
                Err(error) => {
                    return error.emit_as_expr_tokens().into();
                }
                _ => {}
            };

            body.extend(quote! {
                #[cfg(feature = #feature)]
                match #handler_ident::handle(&mut input) {
                    Err(error) => match &error {
                        #handler_ident::Error::Incomplete => {}
                        _ => {
                            eprintln!("error handling {} input: {}", stringify!(#handler), error);
                            code = 1;
                        }
                    },
                    _ => {}
                };
            });

            let formatted = unicode::to_upper_camel_case(feature);
            let formatted = format_ident!("{}", formatted);

            error_kinds.extend(quote! {
                #[cfg(feature = #feature)]
                #formatted(#handler_ident::Error),
            });

            error_formatters.extend(quote! {
                #[cfg(feature = #feature)]
                Error::#formatted(error) => error.fmt(f),
            });

            error_sourcers.extend(quote! {
                #[cfg(feature = #feature)]
                Error::#formatted(error) => Some(error),
            });

            error_transformers.extend(quote! {
                #[cfg(feature = #feature)]
                impl From<#handler_ident::Error> for Error {
                    fn from(error: #handler_ident::Error) -> Self {
                        Self::#formatted(error)
                    }
                }
            });
        }
    }

    TokenStream::from(quote! {
        fn main() {
            let start = std::time::Instant::now();

            let mut input = std::env::args().skip(1).peekable();
            let mut code = 0;
            let mut after: Option<Box<dyn Fn()>> = None;

            #before

            log::info!("starting execution");

            #body

            log::info!("{:?} elapsed", start.elapsed());

            if let Some(after) = after {
                after();
            }

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
                }
            }
        }

        impl std::error::Error for Error {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                match self {
                    #error_sourcers
                }
            }
        }

        #error_transformers
    })
}

fn has_before(library_path: &Path) -> Result<bool, Diagnostic> {
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

    Ok(tree
        .items
        .iter()
        .find(|item| match item {
            syn::Item::Fn(ref function) if "before" == &function.sig.ident.to_string()[..] => true,
            _ => false,
        })
        .is_some())
}

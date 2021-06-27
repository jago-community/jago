#[derive(Debug)]
enum Error {
    Syn(syn::Error),
    StdIo(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syn(error) => write!(f, "syn: {}", error),
            Self::StdIo(error) => write!(f, "syn: {}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Syn(error) => Some(error),
            Self::StdIo(error) => Some(error),
        }
    }
}

impl From<syn::Error> for Error {
    fn from(error: syn::Error) -> Self {
        Self::Syn(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::StdIo(error)
    }
}

#[test]
#[ignore]
fn this_test_must_be_defined_directly_below_error_and_above_functionality() {
    use quote::ToTokens;
    use std::io::BufRead;
    let source = include_str!("./lib.rs");
    let reader = std::io::BufReader::new(source.as_bytes());

    let want = reader
        .lines()
        .take(26)
        .fold(String::new(), |mut collected, line| {
            collected.push_str(&line.unwrap());
            collected.push_str("\n");
            collected
        });

    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("lib.rs");

    let input = format!("{}", path.display());
    let mut tokens = proc_macro2::TokenStream::new();
    input.to_tokens(&mut tokens);

    let got = derive_error(tokens).unwrap();
    let got = format!("{}", got);

    assert_eq!(got, want);
}

/// The goal of this library is to provide incredibly simple error handling. First iteration will
/// allow defining errors as such:
///
/// ```compile_error
/// # {
/// author::error!(std::io::Error, custom::Error)
/// # }
/// ```
///
/// The ideal end result will derive the error from source by walking source code to determine
/// required errors.
use proc_macro2::TokenStream;
use quote::ToTokens;

fn derive_error(input: TokenStream) -> Result<TokenStream, Error> {
    use std::io::Read;

    let parsed = syn::parse2::<syn::LitStr>(input)?;

    let file = std::fs::File::open(parsed.value())?;
    let mut reader = std::io::BufReader::new(file);

    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;

    let tree = syn::parse_file(&buffer)?;

    let sources = tree
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Use(use_item) => Some(vec![use_item]),
            syn::Item::Fn(fn_item) => Some(
                fn_item
                    .block
                    .stmts
                    .iter()
                    .filter_map(|statement| match statement {
                        syn::Stmt::Item(item) => Some(item),
                        _ => None,
                    })
                    .filter_map(|item| match item {
                        syn::Item::Use(use_item) => Some(use_item),
                        _ => None,
                    })
                    .collect(),
            ),
            _ => None,
        })
        .flatten();

    for s in sources {
        match &s.tree {
            syn::UseTree::Path(i) => println!("path {}", i.to_token_stream().to_string()),
            syn::UseTree::Name(i) => println!("name {}", i.to_token_stream().to_string()),
            syn::UseTree::Rename(i) => println!("rename {}", i.to_token_stream().to_string()),
            syn::UseTree::Glob(i) => println!("glob {}", i.to_token_stream().to_string()),
            syn::UseTree::Group(i) => println!("group {}", i.to_token_stream().to_string()),
        };
    }

    let tries = tree
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Fn(fn_item) => Some(fn_item),
            _ => None,
        })
        .flat_map(|fn_item| fn_item.block.stmts.iter().cloned())
        .filter_map(|statement| match statement {
            syn::Stmt::Item(_) => None,
            syn::Stmt::Local(local) => local.init.map(|(_, expression)| expression),
            syn::Stmt::Expr(expression) => Some(Box::new(expression)),
            syn::Stmt::Semi(expression, _) => Some(Box::new(expression)),
        })
        .filter_map(|expression| match expression.as_ref() {
            syn::Expr::Try(try_expression) => Some(try_expression.clone()),
            _ => None,
        });

    for s in tries {
        println!("{:?}", s.question_token.spans[0].start());
        println!("{}", s.to_token_stream().to_string());
    }

    unimplemented!()

    // Walk into `tries[n].expr` to determine type
    // Or, use rust-analyzer
    //
    // Use `sources` to navigate to source of import.
    //   std is in ~/.rustup/toolchains/(stable|nightly)-(target-triple:x86_64-apple-darwin)/lib/rustlib/src/rust/library/std/src/...
    //   crates are in ~/.cargo/registry/src/github.com-(hash)/(crate)-(version)/src/...
}

use quote::quote;

#[test]
fn test_derive_from_fields() {
    let want = quote! {
        #[derive(Debug)]
        enum Error {
            Incomplete,
            StdIoError(std::io::Error),
            BadInput(String),
            Shutdown,
        }

        impl std::fmt::Display for Error {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::Incomplete => write!(f, "{}", stringify!(Incomplete)),
                    Self::StdIoError(error) => write!(f, "{}: {}", stringify!(std::io::Error), error),
                    Self::BadInput(value) => write!(f, "{}: {}", stringify!(BadInput), value),
                    Self::Shutdown => write!(f, "{}: {}", stringify!(Shutdown), "Unknown error happened while shutting down server. You can probably ignore this."),
                }
            }
        }

        impl std::error::Error for Error {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                match self {
                    Self::Incomplete => None,
                    Self::StdIoError(error) => Some(error),
                    Self::BadInput(_) => None,
                    Self::Shutdown => None,
                }
            }
        }

        impl From<std::io::Error> for Error {
            fn from(error: std::io::Error) -> Self {
                Self::StdIoError(error)
            }
        }
    };

    let input = quote! {
        Incomplete,
        std::io::Error,
        BadInput(String),
        (Shutdown, "Unknown error happened while shutting down server. You can probably ignore this."),
    };

    let got = derive_from_fields(input).map_err(|h| dbg!(h)).unwrap();

    pretty_assertions::assert_eq!(got.to_string(), want.to_string());
}

fn derive_from_fields(input: TokenStream) -> Result<TokenStream, Error> {
    let input = syn::parse2::<Kinds>(input)?;

    let mut error_kinds = quote!();
    let mut error_formatters = quote!();
    let mut error_sourcers = quote!();
    let mut error_transformers = quote!();

    for kind in input.inner() {
        let path = &kind.path;

        let (formatted, is_identity) = match path.get_ident() {
            Some(identity) => (format!("{}", identity), true),
            None => (
                path.segments
                    .iter()
                    .map(|segment| utility::unicode::upper_first(&format!("{}", segment.ident)))
                    .collect::<String>(),
                false,
            ),
        };

        let formatted = quote::format_ident!("{}", formatted);

        error_kinds.extend(match (is_identity, &kind.data) {
            (true, Some(data)) => quote! {
                #formatted(#data),
            },
            (true, None) => quote! {
                #formatted,
            },
            (false, Some(data)) => quote! {
                #formatted(#path, #data),
            },
            (false, None) => quote! {
                #formatted(#path),
            },
        });

        error_formatters.extend(match (is_identity, kind.data.is_some(), &kind.message) {
            (true, true, Some(message)) => quote! {
                Self::#formatted(value) => write!(f, "{} ({}): {}", stringify!(#path), #message, value),
            },
            (true, false, Some(message)) => quote! {
                Self::#formatted => write!(f, "{}: {}", stringify!(#path), #message),
            },
            (false, true, Some(message)) => quote! {
                Self::#formatted(error, value) => write!(f, "{} ({}) [{}]: {}", stringify!(#path), value, #message, error),
            },
            (false, false, Some(message)) => quote! {
                Self::#formatted(error) => write!(f, "{} [{}]: {}", stringify!(#path), #message, error),
            },
            (true, true, None) => quote! {
                Self::#formatted(value) => write!(f, "{}: {}", stringify!(#path), value),
            },
            (true, false, None) => quote! {
                Self::#formatted => write!(f, "{}", stringify!(#path)),
            },
            (false, true, None) => quote! {
                Self::#formatted(error, value) => write!(f, "{} ({}): {}", stringify!(#path), value, error),
            },
            (false, false, None) => quote! {
                Self::#formatted(error) => write!(f, "{}: {}", stringify!(#path), error),
            },
        });

        error_sourcers.extend(match (is_identity, kind.data.is_some()) {
            (true, true) => quote! {
                Self::#formatted(_) => None,
            },
            (true, false) => quote! {
                Self::#formatted => None,
            },
            (false, true) => quote! {
                Self::#formatted(error, _) => Some(error),
            },
            (false, false) => quote! {
                Self::#formatted(error) => Some(error),
            },
        });

        if !is_identity {
            error_transformers.extend(quote! {
                impl From<#path> for Error {
                    fn from(error: #path) -> Self {
                        Self::#formatted(error)
                    }
                }
            });
        }
    }

    Ok(TokenStream::from(quote! {
        #[derive(Debug)]
        pub enum Error {
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
    }))
}

use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Path, Token, Type,
};

struct Kind {
    path: Path,
    message: Option<String>,
    data: Option<Type>,
}

struct Kinds(Punctuated<Kind, token::Comma>);

impl Kinds {
    fn inner(&self) -> impl Iterator<Item = &Kind> {
        self.0.iter()
    }
}

impl Parse for Kinds {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut kinds = Punctuated::new();
        while !input.is_empty() {
            if input.peek(token::Paren) {
                let content;
                let _wrapper = syn::parenthesized!(content in input);
                let path: Path = content.parse()?;
                let _punct: Token![,] = content.parse()?;
                let literal: syn::LitStr = content.parse()?;

                kinds.push(Kind {
                    path,
                    message: Some(literal.value()),
                    data: None,
                });
            } else if input.peek(syn::Ident) {
                let path: Path = input.parse()?;
                let mut data = None;

                if input.peek(token::Paren) {
                    let content;
                    let _wrapper = syn::parenthesized!(content in input);
                    if let Ok(data_type) = content.parse() {
                        data = Some(data_type);
                    }
                }

                kinds.push(Kind {
                    path,
                    message: None,
                    data,
                });
            } else {
                let path: Path = input.parse()?;

                kinds.push(Kind {
                    path,
                    message: None,
                    data: None,
                });
            }

            if input.is_empty() {
                break;
            }
            let punct: Token![,] = input.parse()?;
            kinds.push_punct(punct);
        }
        Ok(Kinds(kinds))
    }
}

use proc_macro2_diagnostics::{Diagnostic, Level};

#[proc_macro]
pub fn error(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match derive_from_fields(proc_macro2::TokenStream::from(input)) {
        Ok(derived) => derived.into(),
        Err(error) => Diagnostic::new(Level::Error, "Could not derive string literal from input.")
            .error(error.to_string())
            .emit_as_expr_tokens()
            .into(),
    }
}

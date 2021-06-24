#[derive(Debug)]
enum Error {
    Syn(syn::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syn(error) => write!(f, "syn: {}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Syn(error) => Some(error),
        }
    }
}

impl From<syn::Error> for Error {
    fn from(error: syn::Error) -> Self {
        Self::Syn(error)
    }
}

#[test]
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

    println!("{}", want);

    let input = "libary/author/src/lib.rs";
    let mut tokens = proc_macro2::TokenStream::new();
    input.to_tokens(&mut tokens);

    let got = derive_error(tokens).unwrap();
    let got = format!("{}", got);

    assert_eq!(got, want);
}

use proc_macro2::TokenStream;

fn derive_error(input: TokenStream) -> Result<TokenStream, Error> {
    let parsed = syn::parse2::<syn::LitStr>(input)?;

    unimplemented!()
}

use proc_macro2_diagnostics::{Diagnostic, Level};

#[proc_macro]
pub fn error(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match derive_error(proc_macro2::TokenStream::from(input)) {
        Ok(derived) => derived.into(),
        Err(error) => Diagnostic::new(Level::Error, "Could not derive string literal from input.")
            .error(error.to_string())
            .emit_as_expr_tokens()
            .into(),
    }
}

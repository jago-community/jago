mod error;

#[derive(Debug)]
enum Error {
    Error(error::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error(error) => write!(f, "error: {}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Error(error) => Some(error),
        }
    }
}

impl From<error::Error> for Error {
    fn from(error: error::Error) -> Self {
        Self::Error(error)
    }
}

use proc_macro2_diagnostics::{Diagnostic, Level};

#[proc_macro]
pub fn error(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match error::derive_from_fields(proc_macro2::TokenStream::from(input)) {
        Ok(derived) => derived.into(),
        Err(error) => Diagnostic::new(Level::Error, "Could not derive string literal from input.")
            .error(error.to_string())
            .emit_as_expr_tokens()
            .into(),
    }
}

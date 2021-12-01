mod essay;
mod program;

use proc_macro2_diagnostics::{Diagnostic, Level};

#[proc_macro]
pub fn essay(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match essay::derive(proc_macro2::TokenStream::from(input)) {
        Ok(derived) => derived.into(),
        Err(error) => Diagnostic::new(Level::Error, "Could not derive string literal from input.")
            .error(error.to_string())
            .emit_as_expr_tokens()
            .into(),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("StdIoError {0}")]
    StdIoError(std::io::Error),
    #[error("BadInput")]
    BadInput,
    #[error("Unknown error happened while shutting down server. You can probably ignore this.")]
    Shutdown,
}

use proc_macro2::TokenStream;
use quote::ToTokens;

pub fn derive(_: TokenStream) -> Result<TokenStream, Error> {
    let source = "";
    let mut target = TokenStream::new();
    source.to_tokens(&mut target);
    Ok(target)
}

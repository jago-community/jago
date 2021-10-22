use crate::Context;

use std::iter::Peekable;

pub fn handle(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &mut Context,
) -> Result<(), Error> {
    match input.peek() {
        Some(next) if &next[..] == "browse" => browse(input, context),
        _ => Err(Error::Incomplete),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("NoCache")]
    NoCache,
}

use binary_install::Cache;
use wasm_pack::{install::InstallMode, test::webdriver::get_or_install_geckodriver};

pub fn browse(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &mut Context,
) -> Result<(), Error> {
    let cache = dirs::cache_dir().map_or(Err(Error::NoCache), |cache| {
        Cache::at(cache.join("browsers"))
    })?;

    let driver = get_or_install_geckodriver(&cache);

    unimplemented!()
}

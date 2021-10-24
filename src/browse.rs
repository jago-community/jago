use crate::Context;

use std::iter::Peekable;

pub fn handle(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &mut Context,
) -> Result<(), Error> {
    match input.peek() {
        Some(next) if &next[..] == "browse" => browse(input, context),
        _ => Ok(()),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("NoCache")]
    NoCache,
    #[error("NoData")]
    NoData,
    #[error("NoHome")]
    NoHome,
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
}

use std::{io::Write, path::PathBuf};

fn browse(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &mut Context,
) -> Result<(), Error> {
    let host_manifest = dirs::data_dir().map_or(Err(Error::NoData), |data| {
        Ok(data
            .join("Mozilla")
            .join("NativeMessagingHosts")
            .join("jago.json"))
    })?;

    if !host_manifest.exists() {
        let binary_location = PathBuf::from(env!("CARGO_HOME"))
            .join("bin")
            .join("cargo-jago");

        let mut file = std::fs::File::create(&host_manifest)?;

        write!(
            file,
            r#"{{
    "name": "jago",
    "description": "Interact with anarchy.",
    "path": "{}",
    "type": "stdio",
    "allowed_extensions": [ "wasm@jago.community" ]
}}
"#,
            binary_location.display()
        )?;
    }

    Ok(())
}

/*use binary_install::Cache;
use wasm_pack::{install::InstallMode, test::webdriver::get_or_install_geckodriver};

fn browse(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &mut Context,
) -> Result<(), Error> {
    let cache = dirs::cache_dir().map_or(Err(Error::NoCache), |cache| {
        Cache::at(cache.join("browsers"))
    })?;

    let driver = get_or_install_geckodriver(&cache);

    unimplemented!()
}*/

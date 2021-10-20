use crate::Context;

use std::iter::Peekable;

pub fn handle(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &mut Context,
) -> Result<(), Error> {
    match input.peek() {
        Some(next) if &next[..] == "serve" => serve(input, context),
        _ => Ok(()),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("NoHome")]
    NoHome,
    #[error("WasmPack: {0}")]
    WasmPack(failure::Error),
    #[error("InputOutput: {0}")]
    InputOutput(#[from] std::io::Error),
    #[error("StripPath {0}")]
    StripPath(#[from] std::path::StripPrefixError),
}

use tokio::runtime::Runtime;

fn serve(
    _input: &mut Peekable<impl Iterator<Item = String>>,
    context: &mut Context,
) -> Result<(), Error> {
    let runtime = Runtime::new()?;

    let home = dirs::home_dir().map_or(Err(Error::NoHome), Ok)?;
    let source = home
        .join("local")
        .join("jago")
        .join("crates")
        .join("wasm")
        .join("target")
        .join("pack")
        .join("web");

    #[cfg(feature = "logs")]
    log::info!("serving from {}", source.display());

    runtime.block_on(async {
        warp::serve(warp::fs::dir(source))
            .run(([127, 0, 0, 1], 3030))
            .await;

        Ok(())
    })
}

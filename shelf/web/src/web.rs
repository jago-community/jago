#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("SetupLog")]
    SetupLog,
    #[error("External")]
    External(wasm_bindgen::JsValue),
    #[error("NoWindow")]
    NoWindow,
    #[error("NoDocument")]
    NoDocument,
    #[error("NoBody")]
    NoBody,
    #[error("Life")]
    Handle(#[from] crate::handle::Error),
    //#[error("Life")]
    //Life(#[from] crate::life::Error),
}

use seed::{prelude::*, *};

pub fn handle(key: &str) -> Result<(), Error> {
    //crate::life::handle(key)?;

    crate::handle::handle(key)?;

    Ok(())
}

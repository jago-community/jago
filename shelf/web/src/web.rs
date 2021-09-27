#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("SetupLog")]
    SetupLog,
    #[error("External")]
    External(wasm_bindgen::JsValue),
    #[error("NoWindow")]
    NoWindow,
    #[error("Handle")]
    Handle(#[from] crate::handle::Error),
    #[error("Life")]
    Life(#[from] crate::life::Error),
}

use seed::{prelude::*, *};

pub fn handle(key: &str) -> Result<(), Error> {
    let window = web_sys::window().map_or(Err(Error::NoWindow), Ok)?;
    let location = window.location();
    let path = location.pathname().map_err(Error::External)?;

    if path.ends_with("life.html") {
        crate::life::handle(key)?;
    } else {
        crate::handle::handle(key)?;
    }

    Ok(())
}

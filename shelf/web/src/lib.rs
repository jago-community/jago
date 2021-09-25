mod handle; // life;
mod tree;
mod web;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("External {0:?}")]
    External(wasm_bindgen::JsValue),
}

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn handle() -> Result<(), JsValue> {
    console_log::init_with_level(log::Level::Debug)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;

    #[cfg(feature = "popup")]
    let output = web::handle("jago").map_err(|error| JsValue::from_str(&error.to_string()));

    #[cfg(feature = "background")]
    let output = {
        log::info!("background");
        Ok(())
    };

    #[cfg(not(any(feature = "popup", feature = "background")))]
    let output = Ok(());

    log::info!("{:?}", output);

    output
}

#[wasm_bindgen]
pub fn consume(key: &str, input: JsValue) {
    log::info!("consume: {} -> {:?}", key, input);
}

#[wasm_bindgen]
pub fn dismantle(input: web_sys::Node, handle: &js_sys::Function) -> Result<(), JsValue> {
    encyclopedia::index(&input, |to_index| {
        handle
            .call1(&JsValue::NULL, &JsValue::from_str(to_index))
            .map(|_| ())
            .map_err(Error::External)
    })
    .map_err(|error| JsValue::from_str(&error.to_string()))
}

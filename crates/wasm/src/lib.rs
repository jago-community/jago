use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen(start)]
pub fn handle() -> Result<(), JsValue> {
    console_log::init_with_level(log::Level::Debug)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;

    log::info!("Hello, test-wasm!");

    Ok(())
}

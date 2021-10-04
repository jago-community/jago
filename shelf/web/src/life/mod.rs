pub mod context;
//mod draw;

//book::error!(draw::Error, context::Error, crate::tree::Error);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("External")]
    External(wasm_bindgen::JsValue),
    #[error("Tree")]
    Tree(#[from] crate::tree::Error),
    #[error("Context")]
    Context(#[from] context::Error),
    //#[error("Draw")]
    //Draw(#[from] draw::Error),
}

use context::Universe;

use std::sync::Arc;

pub fn handle(key: &str) -> Result<(), Error> {
    /*    let (_, body) = crate::tree::context()?;

    body.set_attribute("style", "margin: 0px;")
        .map_err(Error::External)?;

    let context = Arc::new(Context::from_width_height(64, 64));

    let set = crate::tree::roots(&format!("canvas.{}", key))?;

    for tree in set {
        draw::handle(&tree, context.clone())?;
    }*/

    Ok(())
}

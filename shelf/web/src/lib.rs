mod handle;
pub mod life;
mod tree;
mod web;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("External {0:?}")]
    External(wasm_bindgen::JsValue),
    #[error("Conversion")]
    Conversion,
    #[error("NoHead")]
    NoHead,
    #[error("NoBody")]
    NoBody,
    #[error("UnknownNodeType {0}")]
    UnknownNodeType(u16),
    #[error("NoChildAt {0}")]
    NoChildAt(u32),
    #[error("NoWindow")]
    NoWindow,
    #[error("NoLocation")]
    NoLocation,
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
pub fn dismantle(input: web_sys::Node, handle: &js_sys::Function) -> Result<(), JsValue> {
    dismantle_node(&input, handle).map_err(|error| JsValue::from_str(&error.to_string()))
}

use wasm_bindgen::JsCast;

fn dismantle_node(input: &web_sys::Node, handle: &js_sys::Function) -> Result<(), Error> {
    match input.node_type() {
        web_sys::Node::DOCUMENT_NODE => {
            let document = input
                .dyn_ref::<web_sys::Document>()
                .map_or(Err(Error::Conversion), Ok)?;
            let location = document.location().map_or(Err(Error::NoLocation), Ok)?;
            let value: JsValue = location
                .dyn_into()
                .ok()
                .map_or(Err(Error::Conversion), Ok)?;
            handle
                .call1(&JsValue::NULL, &value)
                .map(|_| ())
                .map_err(Error::External)?;
            // TODO: do what default case does right now too
            Ok(())
        }
        web_sys::Node::TEXT_NODE => {
            if let Some(text) = input.text_content() {
                handle
                    .call1(&JsValue::NULL, &JsValue::from_str(&text))
                    .map(|_| ())
                    .map_err(Error::External)
            } else {
                Ok(())
            }
        }
        _ => {
            let children = input.child_nodes();

            for index in 0..children.length() {
                let child = children
                    .get(index)
                    .map_or(Err(Error::NoChildAt(index)), Ok)?;
                dismantle_node(&child, handle)?;
            }

            Ok(())
        }
    }

    /*match input.node_type() {
        web_sys::Node::DOCUMENT_NODE => {
            let document: &web_sys::Document =
                input.dyn_ref().map_or(Err(Error::Conversion), Ok)?;

            let head = document.head().map_or(Err(Error::NoHead), Ok)?;
            dismantle_node(head.as_ref(), handle)?;

            let body = document.body().map_or(Err(Error::NoBody), Ok)?;
            dismantle_node(body.as_ref(), handle)
        }
        node_type @ _ => Err(Error::UnknownNodeType(node_type)),
    }*/

    /*
    encyclopedia::index(&input, |to_index| {
        handle
            .call1(&JsValue::NULL, &JsValue::from_str(to_index))
            .map(|_| ())
            .map_err(Error::External)
    })
    .map_err(|error| JsValue::from_str(&error.to_string()))*/
}

pub use life::context::{Cell, Context};

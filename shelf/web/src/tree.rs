#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("External")]
    External(wasm_bindgen::JsValue),
    #[error("NoWindow")]
    NoWindow,
    #[error("NoDocument")]
    NoDocument,
    #[error("NoBody")]
    NoBody,
    #[error("Incomplete")]
    Incomplete,
    #[error("Conversion error between external items")]
    Conversion,
}

use wasm_bindgen::JsCast;

fn context() -> Result<(web_sys::Document, web_sys::HtmlElement), Error> {
    let window = web_sys::window().map_or(Err(Error::NoWindow), Ok)?;
    let document = window.document().map_or(Err(Error::NoDocument), Ok)?;
    let body = document.body().map_or(Err(Error::NoBody), Ok)?;

    Ok((document, body))
}

pub fn roots(key: &str) -> Result<Vec<web_sys::Element>, Error> {
    let (document, body) = context()?;

    let set = body.query_selector_all(key).map_err(Error::External)?;

    let mut output = vec![];

    if set.length() == 0 {
        let added = append_selector(&document, &body, key)?;
        output.push(added);
    }

    Ok(output)
}

/// Crate for now assumes `div.some-key` or `canvas.some-key`
fn append_selector(
    document: &web_sys::Document,
    target: &web_sys::Node,
    selector: &str,
) -> Result<web_sys::Element, Error> {
    let mut parts = selector.split(".");
    let kind = parts.next().map_or(Err(Error::Incomplete), Ok)?;
    let portal = document.create_element(kind).map_err(Error::External)?;

    for key in parts {
        portal.set_class_name(key);
    }

    target.append_child(&portal).map_err(Error::External)?;

    Ok(portal)
}

use crate::web;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(u8)]
#[derive(PartialEq, Eq)]
pub enum Setting {
    Content = 0,
    Handle = 1,
    Background = 2,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Web {0}")]
    Web(#[from] web::Error),
}

pub fn handle() -> Result<(), Error> {
    handle_context()
}

#[cfg(feature = "content")]
fn handle_context() -> Result<(), Error> {
    log::info!("content");
    // Dismantle Document
    // Put parts in crdts::GSet
    // Broadcast crdts::GSet
    Ok(())
}

#[cfg(feature = "popup")]
fn handle_context() -> Result<(), Error> {
    web::handle("jago").map_err(Error::from)
}

#[cfg(feature = "background")]
fn handle_context() -> Result<(), Error> {
    // Listen to messages being sent
    // Add messages to crdts::GSet
    // Index and save content in crdts::GSet to system
    log::info!("background");
    Ok(())
}

#[cfg(not(any(feature = "content", feature = "popup", feature = "background")))]
fn handle_context() -> Result<(), Error> {
    unreachable!()
}

use crdts::{CmRDT, CvRDT, GSet};

#[wasm_bindgen]
pub struct Context {
    keys: GSet<String>,
}

#[wasm_bindgen]
impl Context {
    pub fn empty() -> Context {
        Context { keys: GSet::new() }
    }

    pub fn wrap(&mut self, _setting: Setting, key: String) {
        self.keys.insert(key);
    }
}

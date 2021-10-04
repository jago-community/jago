#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
}

#[cfg(target = "wasm32-unknown-unknown")]
use wasm_bindgen::prelude::*;

use crdts::{CmRDT, CvRDT, GSet};

#[derive(Debug)]
#[cfg_attr(target = "wasm32-unknown-unknown", wasm_bindgen)]
pub struct Puzzle {
    keys: GSet<String>,
}

#[cfg_attr(target = "wasm32-unknown-unknown", wasm_bindgen)]
impl Puzzle {
    pub fn empty() -> Self {
        Self { keys: GSet::new() }
    }

    pub fn wrap(&mut self, key: String) {
        self.keys.insert(key);
    }
}

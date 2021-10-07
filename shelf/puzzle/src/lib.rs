mod puzzle;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("Serialize {0}")]
    Serialize(#[from] bincode::Error),
}

#[cfg(target = "wasm32-unknown-unknown")]
use wasm_bindgen::prelude::*;

use crdts::GSet;

#[derive(Debug)]
#[cfg_attr(target = "wasm32-unknown-unknown", wasm_bindgen)]
pub struct Puzzle {
    keys: GSet<Vec<u8>>,
}

use serde::Serialize;

#[cfg_attr(target = "wasm32-unknown-unknown", wasm_bindgen)]
impl Puzzle {
    pub fn empty() -> Self {
        Self { keys: GSet::new() }
    }

    pub fn wrap(&mut self, key: impl Serialize) -> Result<(), Error> {
        let key = bincode::serialize(&key)?;
        self.keys.insert(key);
        Ok(())
    }

    pub fn keys(&self) -> impl Iterator<Item = Vec<u8>> {
        self.keys.read().into_iter()
    }
}

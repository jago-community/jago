#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("Bincode {0}")]
    Bincode(#[from] bincode::Error),
    #[cfg(feature = "web")]
    #[error("External {0:?}")]
    External(wasm_bindgen::JsValue),
    #[cfg(feature = "web")]
    #[error("Conversion")]
    Conversion,
    #[cfg(feature = "web")]
    #[error("NoHead")]
    NoHead,
    #[cfg(feature = "web")]
    #[error("NoBody")]
    NoBody,
    #[cfg(feature = "web")]
    #[error("UnknownNodeType {0}")]
    UnknownNodeType(u16),
    #[cfg(feature = "web")]
    #[error("NoChildAt {0}")]
    NoChildAt(u32),
    #[cfg(feature = "web")]
    #[error("NoWindow")]
    NoWindow,
    #[cfg(feature = "web")]
    #[error("NoLocation")]
    NoLocation,
    #[cfg(feature = "web")]
    #[error("JsValue")]
    JsValue(#[from] serde_json::Error),
}

use std::{
    collections::BTreeSet,
    convert::{TryFrom, TryInto},
};

use serde::{Deserialize, Serialize};

use crdts::{
    merkle_reg::{Hash, MerkleReg, Node},
    CmRDT,
};

pub type Value = Vec<u8>;

pub type Surface = Node<Value>;

#[cfg(feature = "web")]
use wasm_bindgen::{prelude::*, JsCast, JsValue};

#[cfg_attr(feature = "web", wasm_bindgen)]
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Shadow {
    surface: MerkleReg<Value>,
}

impl Shadow {
    fn gather(&self) -> Vec<Value> {
        self.gather_nodes(&self.surface.read().hashes())
    }

    fn gather_nodes(&self, keys: &BTreeSet<Hash>) -> Vec<Value> {
        let mut output = vec![];

        for key in keys {
            if let Some(node) = self.surface.node(*key) {
                output.push(node.value.clone());

                let children = self.gather_nodes(&node.children);

                output.extend_from_slice(children.as_ref());
            }
        }

        output
    }
}

impl TryFrom<Shadow> for String {
    type Error = Error;

    fn try_from(from: Shadow) -> Result<Self, Self::Error> {
        from.gather()
            .iter()
            .map(|slice| bincode::deserialize::<Option<String>>(slice).map_err(Error::from))
            .try_fold(String::new(), |mut so_far, next| {
                let next = next?;
                if let Some(next) = next {
                    so_far.push_str(&next);
                    so_far.push('\n');
                }
                Ok(so_far)
            })
    }
}

#[cfg(feature = "web")]
impl TryInto<Shadow> for web_sys::Node {
    type Error = Error;

    fn try_into(self) -> Result<Shadow, Self::Error> {
        let mut shadow = Shadow::default();
        shadow.cast(self, None)?;
        Ok(shadow)
    }
}

#[cfg(feature = "web")]
#[wasm_bindgen]
impl Shadow {
    pub fn perceive() -> Self {
        Self::default()
    }

    pub fn cover(&mut self, input: JsValue) -> Result<(), JsValue> {
        let (key, children) = input.into_serde()?;

        let operation = self.surface.write(input.value, input.children);

        let output = operation.hash();

        self.surface.apply(operation);

        Ok(())
    }
    // todo fix erros from requiring this looker function
    pub fn cast(
        &mut self,
        input: web_sys::Node,
        looker: &js_sys::Function,
    ) -> Result<Option<Hash>, Error> {
        let mut current = None;

        match input.node_type() {
            web_sys::Node::DOCUMENT_NODE => {
                let document = input
                    .dyn_ref::<web_sys::Document>()
                    .map_or(Err(Error::Conversion), Ok)?;

                // TODO: here
                let location: String = document
                    .location()
                    .map_or(Err(Error::NoLocation), Ok)
                    .and_then(|location| location.href().map_err(Error::External))?;

                current = Some(location);
            }
            web_sys::Node::TEXT_NODE => {
                if let Some(text) = input.text_content() {
                    current = Some(text);
                }
            }
            web_sys::Node::ELEMENT_NODE => {
                let element = input
                    .dyn_ref::<web_sys::Element>()
                    .map_or(Err(Error::Conversion), Ok)?;

                match &element.tag_name()[..] {
                    "script" => {
                        return Ok(None);
                    }
                    _ => {}
                };
            }
            _ => {}
        };

        let child_nodes = input.child_nodes();

        let mut children = BTreeSet::default();

        for index in 0..child_nodes.length() {
            let child = child_nodes
                .get(index)
                .map_or(Err(Error::NoChildAt(index)), Ok)?;

            if let Some(child) = self.cast(child, looker)? {
                children.insert(child);
            }
        }

        let key = bincode::serialize(&current)?;

        if let Some(looker) = looker {
            let message = JsValue::from_serde(&(&key, &children))?;
            looker.call1(&JsValue::NULL, &message);
        }

        let operation = self.surface.write(key, children);

        let output = operation.hash();

        self.surface.apply(operation);

        Ok(Some(output))
    }
}

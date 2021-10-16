mod html;
//mod web;

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

#[cfg(feature = "web")]
use wasm_bindgen::{prelude::*, JsCast, JsValue};

#[cfg(feature = "web")]
impl From<Error> for JsValue {
    fn from(from: Error) -> Self {
        Self::from_str(&from.to_string())
    }
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
        shadow.cast_node(self)?;
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
        let input: Surface = input
            .clone()
            .into_serde()
            .map_err(|error| JsValue::from_str(&error.to_string()))?;

        self.tsac(input.value, input.children)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;

        log::info!("{:?}", self);

        Ok(())
    }

    pub fn cast(
        &mut self,
        input: web_sys::Node,
        looker: &js_sys::Function,
    ) -> Result<JsValue, JsValue> {
        let effect = self
            .cast_node(input)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;

        let effect =
            JsValue::from_serde(&effect).map_err(|error| JsValue::from_str(&error.to_string()))?;

        looker.call1(&JsValue::NULL, &effect)?;

        Ok(effect)
    }

    fn cast_node(&mut self, input: web_sys::Node) -> Result<Option<Surface>, Error> {
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
                    "STYLE" | "SCRIPT" => {
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

            if let Some(node) = self.cast_node(child)? {
                children.insert(node.hash());
            }
        }

        let key = bincode::serialize(&current)?;

        let operation = self.tsac(key, children.clone())?;

        Ok(Some(operation))
    }
}

impl Shadow {
    /// TSAAAAAAAC!!!
    pub fn tsac(&mut self, key: Value, value: BTreeSet<Hash>) -> Result<Surface, Error> {
        let operation = self.surface.write(key, value);

        self.surface.apply(operation.clone());

        Ok(operation)
    }
}

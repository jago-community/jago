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
}

use std::{
    collections::BTreeSet,
    convert::{TryFrom, TryInto},
};

use crdts::{
    merkle_reg::{Hash, MerkleReg},
    CmRDT,
};

pub type Node = Vec<u8>;

#[derive(Default, Debug)]
pub struct Shadow {
    surface: MerkleReg<Node>,
}

impl Shadow {
    fn read(&self) -> Vec<Node> {
        let nodes = self.surface.read().nodes().cloned().collect::<Vec<_>>();
        log::info!("{:?}", nodes);
        self.surface.read().values().cloned().collect()
    }
}

impl TryFrom<Shadow> for String {
    type Error = Error;

    fn try_from(from: Shadow) -> Result<Self, Self::Error> {
        from.read()
            .iter()
            .map(|slice| bincode::deserialize::<Option<String>>(slice).map_err(Error::from))
            .try_fold(String::new(), |mut so_far, next| {
                let next = next?;
                if let Some(next) = next {
                    so_far.push_str(&next);
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
        shadow.cast(self)?;
        Ok(shadow)
    }
}

#[cfg(feature = "web")]
use wasm_bindgen::JsCast;

#[cfg(feature = "web")]
impl Shadow {
    fn cast(&mut self, input: web_sys::Node) -> Result<Hash, Error> {
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
            _ => {}
        };

        let child_nodes = input.child_nodes();

        let mut children = BTreeSet::default();

        for index in 0..child_nodes.length() {
            let child = child_nodes
                .get(index)
                .map_or(Err(Error::NoChildAt(index)), Ok)?;

            let child = self.cast(child)?;

            children.insert(child);
        }

        let key = bincode::serialize(&current)?;

        let operation = self.surface.write(key, children);

        let output = operation.hash();

        self.surface.apply(operation);

        Ok(output)
    }
}

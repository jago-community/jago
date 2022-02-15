#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialize {0}")]
    Serialize(#[from] crate::serialize::Error),
}

use serde::Serialize;

pub trait View: Serialize + Sized {}

impl<'a, S: Serialize> View for S {}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("NoHome")]
    NoHome,
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

use ::{
    crdts::{CmRDT, List},
    std::{
        path::PathBuf,
        sync::{Arc, Mutex},
    },
};

pub struct Context<W> {
    buffer: Arc<Mutex<W>>,
    editor: Arc<Mutex<Editor>>,
}

#[derive(Default)]
pub struct Editor {
    path: PathBuf,
    buffer: List<char, u8>,
}

impl<P: Into<PathBuf>> From<P> for Editor {
    fn from(path: P) -> Editor {
        Editor {
            path: path.into(),
            buffer: Default::default(),
        }
    }
}

pub enum Op {
    Read,
}

impl CmRDT for Editor {
    type Op = Op;
    type Validation = Error;

    fn validate_op(&self, op: &Self::Op) -> Result<(), Self::Validation> {
        Ok(())
    }

    fn apply(&mut self, op: Self::Op) {
        // ...
    }
}

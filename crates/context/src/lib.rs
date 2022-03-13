mod buffer;
mod handle;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Environment {0}")]
    Environment(#[from] environment::Error),
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
    #[error("Encoding {0}")]
    Encoding(#[from] std::string::FromUtf8Error),
}

pub use handle::{Directive, Directives, Handle};

use buffer::Buffer;

use ::{
    crdts::CmRDT,
    instrument::prelude::*,
    std::{fmt::Display, fs::read_dir, io::Read},
};

static DEFAULT_ACTOR: u8 = 0;

pub struct Context {
    buffer: Buffer,
}

impl Context {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
        }
    }
}

pub enum Op {
    Read,
}

impl CmRDT for Context {
    type Op = Op;
    type Validation = Error;

    fn validate_op(&self, op: &Self::Op) -> Result<(), Error> {
        Ok(())
    }

    fn apply(&mut self, op: Self::Op) {
        match op {
            Op::Read => {
                if let Err(error) = self.read() {
                    //let op = self.errors.append(error, self.actor);

                    //self.errors.apply(op);
                }
            }
        }
    }
}

impl Context {
    fn read(&mut self) -> Result<(), Error> {
        let root = environment::copy_directory()?;

        let read = read_dir(root)?;

        for entry in read.filter_map(Result::ok) {
            self.write_row(entry.path().display());
        }

        Ok(())
    }

    fn write_row(&mut self, item: impl Display) {
        // ...
    }
}

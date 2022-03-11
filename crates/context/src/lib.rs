mod context;
mod handle;

pub use handle::{Directive, Directives, Handle};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

use ::{
    crdts::{CmRDT, List},
    std::fmt,
};

pub struct Context {
    buffer: List<char, u8>,
}

impl Context {
    pub fn buffer(&self) -> &List<char, u8> {
        &self.buffer
    }
}

static DEFAULT_ACTOR: u8 = 0;

impl From<&str> for Context {
    fn from(input: &str) -> Self {
        let mut buffer = List::new();

        for c in input.chars() {
            let op = buffer.append(c, DEFAULT_ACTOR);
            buffer.apply(op);
        }

        Self { buffer }
    }
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let buffer = self.buffer.read::<String>();

        f.write_str(&buffer)
    }
}

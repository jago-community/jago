pub mod color_picker;
mod shell;
mod traits;

pub use shell::Shell;
pub use traits::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

pub use traits::{Directive, Op};

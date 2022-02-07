mod color_picker;
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

pub struct StartFresh;

use ::{
    crossterm::terminal::{Clear, ClearType},
    std::fmt,
};

impl Command for StartFresh {
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        Clear(ClearType::All).write_ansi(out)
    }
}

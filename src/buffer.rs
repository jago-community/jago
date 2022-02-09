use std::io::Write;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

use ::{
    crossterm::{
        event::Event,
        event::EventStream,
        terminal::{disable_raw_mode, enable_raw_mode, size},
        ExecutableCommand,
    },
    tokio::runtime::Builder as Runtime,
};

pub trait Buffer: Write {}

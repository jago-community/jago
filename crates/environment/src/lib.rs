#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("NoHome")]
    NoHome,
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

use std::{fs::create_dir_all, path::PathBuf};

pub fn target(suffix: &str, ensure: bool) -> Result<PathBuf, Error> {
    dirs::home_dir()
        .ok_or(Error::NoHome)
        .map(|home| home.join("jago").join("target").join("jago").join(suffix))
        .and_then(|path| {
            if ensure {
                create_dir_all(&path)?
            }

            Ok(path)
        })
}

pub fn component(name: &str) -> Result<PathBuf, Error> {
    dirs::home_dir()
        .ok_or(Error::NoHome)
        .map(|home| home.join("jago").join("crates").join(name))
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("NoHome")]
    NoHome,
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

use std::{fs::create_dir_all, path::PathBuf};

pub fn logs_directory(ensure: bool) -> Result<PathBuf, Error> {
    dirs::home_dir()
        .ok_or(Error::NoHome)
        .map(|home| home.join("jago").join("target").join("jago").join("logs"))
        .and_then(|path| {
            if ensure {
                create_dir_all(&path)?
            }

            Ok(path)
        })
}

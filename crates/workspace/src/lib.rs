#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("NoHome")]
    NoHome,
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
    #[error("NoResourceDirectory")]
    NoResourceDirectory,
}

use std::path::PathBuf;

pub fn source_directory() -> Result<PathBuf, Error> {
    dirs::home_dir().map_or(Err(Error::NoHome), |home| Ok(home.join("jago")))
}

pub fn resource_directory() -> Result<PathBuf, Error> {
    let mut current = std::env::current_exe()?;

    loop {
        if !current.pop() {
            return Err(Error::NoResourceDirectory);
        }

        if current.join("Resources").exists() {
            return Ok(current.join("Resources"));
        } else if current.join("assets").exists() {
            return Ok(current);
        }
    }
}

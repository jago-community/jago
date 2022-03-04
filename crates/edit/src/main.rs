mod screen;

use std::{env, path::PathBuf};

fn main() {
    if let Err(error) = editor() {
        eprintln!("error: {}", error);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

fn editor() -> Result<(), Error> {
    let root = env::current_dir()?;

    let directory = read_directory(&root)?;

    Ok(())
}

use std::path::Path;

fn read_directory(path: &Path) -> Result<Vec<PathBuf>, Error> {
    let directory = std::fs::read_dir(path)?;

    Ok(directory
        .into_iter()
        .flat_map(|result| result.ok())
        .map(|entry| entry.path())
        .collect())
}

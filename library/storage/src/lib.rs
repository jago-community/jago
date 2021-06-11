use std::{
    iter::Peekable,
    path::{Path, PathBuf},
};

pub fn handle<I: Iterator<Item = String>>(input: &mut Peekable<I>) -> Result<(), Error> {
    match input.peek() {
        Some(next) if next == "storage" => input.next(),
        _ => return Err(Error::Incomplete),
    };

    match input.next() {
        Some(action) if &action == "link" => link(input)?,
        _ => return Err(Error::Incomplete),
    };

    Ok(())
}

pub fn link<I: Iterator<Item = String>>(input: &mut Peekable<I>) -> Result<(), Error> {
    let this = input
        .next()
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or(Err(Error::Incomplete))?;
    let to = input
        .next()
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or(Err(Error::Incomplete))?;

    create_link(&this, &to)
}

#[cfg(unix)]
pub fn create_link(target: &Path, destination: &Path) -> Result<(), Error> {
    println!("{} -> {}", target.display(), destination.display());
    std::os::unix::fs::symlink(target, destination).map_err(Error::from)
}

#[cfg(windows)]
pub fn create_link(target: &Path, destination: &Path) -> Result<(), Error> {
    let metadata = std::fs::metadata(target)?;

    if metadata.is_file() {
        std::os::windows::fs::symlink_file(target, destination)
    } else {
        std::os::windows::fs::symlink_dir(target, destination)
    }
    .map_err(Error::from)
}

#[derive(Debug)]
pub enum Error {
    Incomplete,
    Machine(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Incomplete => write!(f, "incomplete input for storage"),
            Self::Machine(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Incomplete => None,
            Self::Machine(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::Machine(error)
    }
}

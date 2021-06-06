use std::{
    io::Write,
    path::{Path, PathBuf},
};

use bytes::Bytes;

pub fn handle(target: &Path, input: &Bytes) -> Result<(), Error> {
    let target = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(target)?;
    let mut writer = std::io::BufWriter::new(target);

    writer.write_all(input.as_ref())?;

    writer.write_all(&[b'\n']).map_err(Error::from)
}

pub fn parse<I: Iterator<Item = String>>(input: &mut I) -> Result<(PathBuf, Bytes), Error> {
    let target = if let Some(target) = input.next() {
        PathBuf::from(target)
    } else {
        return Err(Error::Incomplete);
    };

    let rest = input.collect::<Vec<_>>().join(" ");

    Ok((target, Bytes::from(rest)))
}

#[derive(Debug)]
pub enum Error {
    Machine(std::io::Error),
    Incomplete,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Machine(error) => write!(f, "{}", error),
            Error::Incomplete => write!(f, "Incomplete input."),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Machine(error) => Some(error),
            Error::Incomplete => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Machine(error)
    }
}

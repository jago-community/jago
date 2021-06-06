mod repository;

use crate::address::Address;

pub fn ensure<'a>(input: impl Into<Input<'a>>) -> Result<(), Error> {
    match input.into() {
        Input::Repository(address) => repository::ensure(address).map_err(Error::from),
    }
}

pub enum Input<'a> {
    Repository(&'a Address),
}

impl<'a> Into<Input<'a>> for &'a Address {
    fn into(self) -> Input<'a> {
        Input::Repository(self)
    }
}

#[derive(Debug)]
pub enum Error {
    Machine(std::io::Error),
    Repository(repository::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Machine(error) => write!(f, "{}", error),
            Error::Repository(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Machine(error) => Some(error),
            Error::Repository(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Machine(error)
    }
}

impl From<repository::Error> for Error {
    fn from(error: repository::Error) -> Self {
        Self::Repository(error)
    }
}

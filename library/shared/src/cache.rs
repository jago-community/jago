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

author::error!(std::io::Error, repository::Error);

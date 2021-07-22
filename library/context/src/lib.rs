mod environment;

author::error!(NoHome, environment::Error);

pub fn before() -> Result<Option<Box<dyn Fn()>>, Error> {
    environment::populate()?;

    Ok(None)
}

use std::path::PathBuf;

use lazy_static::lazy_static;

lazy_static! {
    static ref HOME: Option<PathBuf> = dirs::home_dir();
}

pub fn home() -> Result<PathBuf, Error> {
    HOME.clone().map_or(Err(Error::NoHome), Ok)
}

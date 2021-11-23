book::error!(NoHome, crate::workspace::Error);

use std::path::PathBuf;

pub fn frame() -> Result<PathBuf, Error> {
    crate::workspace::source_directory()
        .map(|error| error.join("target"))
        .map_or(Err(Error::NoHome), Ok)
}

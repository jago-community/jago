book::error!(NoHome);

use std::path::PathBuf;

pub fn frame() -> Result<PathBuf, Error> {
    dirs::home_dir().map_or(Err(Error::NoHome), Ok)
}

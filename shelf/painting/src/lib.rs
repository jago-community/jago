book::error!(NoHome);

use std::path::PathBuf;

pub fn frame() -> Result<PathBuf, Error> {
    dirs::home_dir()
        .map(|path| path.join("local").join("jago").join("target"))
        .map_or(Err(Error::NoHome), Ok)
}

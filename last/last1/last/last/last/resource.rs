use std::path::PathBuf;

pub struct Resource {
    path: PathBuf,
    bytes: Option<Vec<u8>>,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Io")]
    Io(#[from] std::io::Error),
}

use std::path::Path;

impl From<&Path> for Resource {
    fn from(path: &Path) -> Self {
        Self {
            path: path.into(),
            bytes: None,
        }
    }
}

// use std::{fs::OpenOptions, io::Read, path::Path};

impl Resource {
    pub fn bytes(&self) -> &[u8] {
        unimplemented!()
    }
}

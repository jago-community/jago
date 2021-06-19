use std::{collections::HashMap, io::Read, path::PathBuf};

use serde::{Deserialize, Serialize};

pub fn inspect<P: Into<PathBuf>>(library: P) -> Result<Library, Error> {
    let manifest = library.into().join("Cargo.toml");

    let file = std::fs::File::open(manifest)?;
    let mut reader = std::io::BufReader::new(file);
    let mut configuration = String::new();
    reader.read_to_string(&mut configuration)?;

    toml::from_str(&configuration).map_err(Error::from)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Library {
    features: HashMap<String, Vec<String>>,
    dependencies: HashMap<String, Dependency>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Dependency {
    Version(String),
    Specification(DependencySpecification),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DependencySpecification {
    path: String,
    #[serde(default)]
    optional: bool,
}

#[derive(Debug)]
pub enum Error {
    Machine(std::io::Error),
    Configuration(toml::de::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Machine(error) => write!(f, "{}", error),
            Self::Configuration(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Machine(error) => Some(error),
            Self::Configuration(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Machine(error)
    }
}

impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Self {
        Self::Configuration(error)
    }
}

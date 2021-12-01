#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Workspace {0}")]
    Workspace(#[from] workspace::Error),
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
    #[error("Toml {0}")]
    ReadToml(#[from] toml::de::Error),
}

use proc_macro2::TokenStream;
use quote::ToTokens;

pub fn derive(_: TokenStream) -> Result<TokenStream, Error> {
    let source = "";
    let mut target = TokenStream::new();
    source.to_tokens(&mut target);
    Ok(target)
}

use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]
enum Dependency {
    Version(String),
    Spec {
        path: Option<PathBuf>,
        version: Option<String>,
        #[serde(default)]
        optional: bool,
        #[serde(default)]
        features: Vec<String>,
    },
}

#[derive(Deserialize)]
struct Manifest {
    dependencies: Vec<Dependency>,
}

fn read_dependencies() -> Result<(), Error> {
    let manifest_path = workspace::source_directory().map(|path| path.join("Cargo.toml"))?;
    let manifest = std::fs::read_to_string(&manifest_path)?;
    let manifest = toml::from_str::<Manifest>(&manifest)?;

    let handlers = manifest
        .dependencies
        .iter()
        .filter_map(|dependency| match dependency {
            Dependency::Spec {
                path,
                version,
                optional,
                features,
            } => {
                unimplemented!()
            }
            _ => None,
        });

    unimplemented!()
}

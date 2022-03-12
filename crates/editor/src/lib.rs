mod dependency;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Environment {0}")]
    Environment(#[from] environment::Error),
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
    #[error("Io {0}")]
    Deserialize(#[from] toml::de::Error),
    #[error("Io {0}")]
    Dependency(#[from] dependency::Error),
}

use ::{
    serde::Deserialize,
    std::{fs::File, io::Read, path::PathBuf},
};

use dependency::Dependency;

pub fn before() -> Result<(), Error> {
    let settings = Settings::read()?;

    let target = settings.target()?;

    for definition in settings.editor.dependencies {
        Dependency::from(definition).ensure(&target)?;
    }

    Ok(())
}

#[derive(Deserialize)]
pub struct Settings {
    editor: Editor,
}

#[derive(Deserialize)]
pub struct Editor {
    target: String,
    dependencies: Vec<DependencyDefinition>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum DependencyDefinition {
    Key(String),
    Definition { key: String, version: String },
}

impl Settings {
    pub fn read() -> Result<Self, Error> {
        let root = environment::workspace().map(|path| path.join("jago.toml"))?;

        let mut file = File::open(&root)?;

        let mut buffer = vec![];

        file.read_to_end(&mut buffer)?;

        toml::from_slice(&buffer).map_err(Error::from)
    }

    fn target(&self) -> Result<PathBuf, Error> {
        environment::home()
            .map(|home| home.join(&self.editor.target).join("editor"))
            .map_err(Error::from)
    }
}

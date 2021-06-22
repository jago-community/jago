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

pub fn libraries<P: Into<PathBuf>>(library: P) -> Result<Vec<String>, Error> {
    let context = library.into();

    let mut builder = ignore::WalkBuilder::new(&context);

    builder.add_ignore(context.join(".dockerignore"));
    builder.add_ignore(context.join(".gitignore"));

    let build_context = builder.build();

    let library = inspect(&context)?;

    let workspace = library.workspace.unwrap_or(Default::default());

    let mut patterns = vec![];

    for pattern in workspace.members {
        let pattern = glob::Pattern::new(&pattern)?;

        patterns.push(pattern);
    }

    let mut output = vec![];

    for entry in build_context {
        let entry = entry?;
        let path = entry.path();

        if path == context {
            continue;
        }

        let path = path.strip_prefix(&context)?;

        if matches(&patterns, path) {
            if let Some(stem) = path.file_stem() {
                if let Some(name) = stem.to_str() {
                    output.push(name.to_string());
                }
            }
        }
    }

    Ok(output)
}

fn matches(patterns: &[glob::Pattern], path: &std::path::Path) -> bool {
    for pattern in patterns {
        if pattern.matches_path_with(
            path,
            glob::MatchOptions {
                require_literal_separator: true,
                ..Default::default()
            },
        ) {
            return true;
        }
    }

    false
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Library {
    features: HashMap<String, Vec<String>>,
    dependencies: HashMap<String, Dependency>,
    workspace: Option<Workspace>,
}

impl Library {
    pub fn handlers(&self) -> Vec<String> {
        self.dependencies
            .iter()
            .filter(|(_, dependency)| match dependency {
                // INCREMENTALLY: smarter matching. Ideal would be to derive from source
                Dependency::Specification(specification) => specification.optional,
                _ => false,
            })
            .map(|(dependency, _)| dependency)
            .cloned()
            .collect()
    }
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

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Workspace {
    members: Vec<String>,
}

#[derive(Debug)]
pub enum Error {
    Machine(std::io::Error),
    Configuration(toml::de::Error),
    Pattern(glob::PatternError),
    Entry(glob::GlobError),
    Context(ignore::Error),
    Prefix(std::path::StripPrefixError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Machine(error) => write!(f, "{}", error),
            Self::Configuration(error) => write!(f, "{}", error),
            Self::Pattern(error) => write!(f, "{}", error),
            Self::Entry(error) => write!(f, "{}", error),
            Self::Context(error) => write!(f, "{}", error),
            Self::Prefix(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Machine(error) => Some(error),
            Self::Configuration(error) => Some(error),
            Self::Pattern(error) => Some(error),
            Self::Entry(error) => Some(error),
            Self::Context(error) => Some(error),
            Self::Prefix(error) => Some(error),
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

impl From<glob::PatternError> for Error {
    fn from(error: glob::PatternError) -> Self {
        Self::Pattern(error)
    }
}

impl From<glob::GlobError> for Error {
    fn from(error: glob::GlobError) -> Self {
        Self::Entry(error)
    }
}

impl From<ignore::Error> for Error {
    fn from(error: ignore::Error) -> Self {
        Self::Context(error)
    }
}

impl From<std::path::StripPrefixError> for Error {
    fn from(error: std::path::StripPrefixError) -> Self {
        Self::Prefix(error)
    }
}

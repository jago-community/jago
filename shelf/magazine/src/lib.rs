book::error!(
    Incomplete,
    std::io::Error,
    toml::de::Error,
    glob::PatternError,
    glob::GlobError,
    ignore::Error,
    std::path::StripPrefixError,
    NoStem,
    WeirdPath,
);

use std::{
    collections::HashMap,
    io::{Read, Write},
    iter::Peekable,
    path::PathBuf,
};

pub fn handle<I: Iterator<Item = String>>(input: &mut Peekable<I>) -> Result<(), Error> {
    match input.peek() {
        Some(next) if next == "manifest-path" => {
            let _ = input.next();

            let path = input.next().map_or(Err(Error::Incomplete), Ok)?;

            let path = PathBuf::from(&path)
                .ancestors()
                .find(|path| path.join("Cargo.toml").is_file())
                .map(PathBuf::from)
                .map_or(Err(Error::Incomplete), Ok)?;

            let output = path.join("Cargo.toml").display().to_string();

            std::io::stdout()
                .write_all(output.as_bytes())
                .map_err(Error::from)
        }
        _ => Err(Error::Incomplete),
    }
}

use serde::{Deserialize, Serialize};

pub fn inspect<P: Into<PathBuf>>(concept: P) -> Result<Concept, Error> {
    let manifest = concept.into().join("Cargo.toml");

    let file = std::fs::File::open(manifest)?;
    let mut reader = std::io::BufReader::new(file);
    let mut configuration = String::new();
    reader.read_to_string(&mut configuration)?;

    toml::from_str(&configuration).map_err(Error::from)
}

pub fn concepts<P: Into<PathBuf>>(context: P) -> Result<Vec<String>, Error> {
    let context = context.into();

    let mut builder = ignore::WalkBuilder::new(&context);

    builder.add_ignore(context.join(".dockerignore"));
    builder.add_ignore(context.join(".gitignore"));

    let build_context = builder.build();

    let concept = inspect(&context)?;

    let workspace = concept.workspace.unwrap_or(Default::default());

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
pub struct Concept {
    features: HashMap<String, Vec<String>>,
    dependencies: HashMap<String, Dependency>,
    workspace: Option<Workspace>,
}

impl Concept {
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

use std::{iter::Peekable, path::Path};

use serde::Serialize;

pub fn handle<I: Iterator<Item = String>>(input: &mut Peekable<I>) -> Result<(), Error> {
    match input.peek() {
        Some(next) if next == "platform" => input.next(),
        _ => return Err(Error::Incomplete),
    };

    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async {
        match input.next() {
            Some(action) => match &action[..] {
                "build" => build(input).await,
                _ => Err(Error::BadInput([action, input.collect()].join(" "))),
            },
            _ => Err(Error::Incomplete),
        }
    })
}

async fn build<I: Iterator<Item = String>>(_input: &mut I) -> Result<(), Error> {
    use bollard::image::BuildImageOptions;
    use bollard::Docker;
    use futures_util::stream::StreamExt;

    let context = dirs::home_dir().unwrap().join("local").join("jago");

    let libraries = library::libraries(&context)?;

    let build_context = get_build_context(&context, &libraries)?;

    let docker = Docker::connect_with_unix_defaults()?;

    let options = BuildImageOptions {
        dockerfile: "definition",
        t: "builder",
        ..Default::default()
    };

    let mut output = docker.build_image(options, None, Some(hyper::Body::from(build_context)));

    while let Some(entry) = output.next().await {
        let entry = entry?;
        if let Some(message) = entry.stream {
            print!("{}", message);
        }
    }

    Ok(())
}

#[test]
fn its_all_about_the_context() {
    let context = dirs::home_dir().unwrap().join("local").join("jago");
    let libraries = library::libraries(&context).unwrap();
    let context = get_build_context(&context, libraries).unwrap();
    let mut archive = tar::Archive::new(&context[..]);
    let context = tempfile::TempDir::new().unwrap();
    let context = context.path();

    archive.unpack(context).unwrap();

    let definition = context.join("definition");
    let mut definition = std::fs::File::open(definition).unwrap();
    let mut buffer = String::new();
    use std::io::Read;
    definition.read_to_string(&mut buffer).unwrap();
    assert!(buffer.starts_with("FROM"));
    assert!(!buffer.contains("{{ endfor }}"));
    assert!(!buffer.contains("html"));
    assert!(buffer.contains("COPY library/library"));
}

fn get_build_context<S: Serialize>(context: &Path, libraries: S) -> Result<Vec<u8>, Error> {
    let mut builder = ignore::WalkBuilder::new(&context);

    builder.add_ignore(context.join(".dockerignore"));
    builder.add_ignore(context.join(".gitignore"));

    let build_context = builder.build();

    let mut archive = tar::Builder::new(vec![]);

    let mut variables = std::collections::HashMap::new();
    let libraries = serde_json::to_value(libraries)?;
    variables.insert("libraries", libraries);

    let definition_path = std::path::PathBuf::from("container")
        .join("builder")
        .join("Dockerfile");

    for entry in build_context {
        let entry = entry?;
        let path = entry.path();

        if path == context {
            continue;
        }

        let path = entry.path();
        let destination = path.strip_prefix(&context)?;

        if destination == &definition_path {
            let mut buffer = vec![];
            shared::source::read_template(&mut buffer, path, &variables)?;
            let mut header = tar::Header::new_gnu();
            header.set_path("definition")?;
            header.set_size(buffer.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            archive.append(&header, &buffer[..]).unwrap();
        } else if let Some(kind) = entry.file_type() {
            if kind.is_file() {
                let mut file = std::fs::File::open(&path)?;
                archive.append_file(destination, &mut file)?;
            } else {
                archive.append_dir(destination, path)?;
            }
        }
    }

    archive.into_inner().map_err(Error::from)
}

#[derive(Debug)]
pub enum Error {
    Incomplete,
    BadInput(String),
    Template(tinytemplate::error::Error),
    Machine(std::io::Error),
    Container(bollard::errors::Error),
    Library(library::Error),
    Context(ignore::Error),
    Prefix(std::path::StripPrefixError),
    Shutdown,
    Serve(server::Error),
    Serialize(serde_json::Error),
    Source(shared::source::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Incomplete => write!(f, "incomplete"),
            Self::BadInput(input) => write!(f, "bad input: {}", input),
            Self::Template(error) => write!(f, "{}", error),
            Self::Machine(error) => write!(f, "{}", error),
            Self::Container(error) => write!(f, "container {}", error),
            Self::Library(error) => write!(f, "{}", error),
            Self::Context(error) => write!(f, "{}", error),
            Self::Prefix(error) => write!(f, "{}", error),
            Self::Shutdown => write!(
                f,
                "Unknown error happened while shutting down server. You can probably ignore this."
            ),
            Self::Serve(error) => write!(f, "{}", error),
            Self::Serialize(error) => write!(f, "{}", error),
            Self::Source(error) => write!(f, "source {}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Incomplete => None,
            Self::BadInput(_) => None,
            Self::Template(error) => Some(error),
            Self::Machine(error) => Some(error),
            Self::Container(error) => Some(error),
            Self::Library(error) => Some(error),
            Self::Context(error) => Some(error),
            Self::Prefix(error) => Some(error),
            Self::Shutdown => None,
            Self::Serve(error) => Some(error),
            Self::Serialize(error) => Some(error),
            Self::Source(error) => Some(error),
        }
    }
}

impl From<tinytemplate::error::Error> for Error {
    fn from(error: tinytemplate::error::Error) -> Self {
        Self::Template(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Machine(error)
    }
}

impl From<bollard::errors::Error> for Error {
    fn from(error: bollard::errors::Error) -> Self {
        Self::Container(error)
    }
}

impl From<library::Error> for Error {
    fn from(error: library::Error) -> Self {
        Self::Library(error)
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

impl From<server::Error> for Error {
    fn from(error: server::Error) -> Self {
        Self::Serve(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialize(error)
    }
}

impl From<shared::source::Error> for Error {
    fn from(error: shared::source::Error) -> Self {
        Self::Source(error)
    }
}

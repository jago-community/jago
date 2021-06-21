use std::{
    iter::Peekable,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::Serialize;
use tinytemplate::TinyTemplate as Templates;

pub fn handle<I: Iterator<Item = String>>(input: &mut Peekable<I>) -> Result<(), Error> {
    match input.peek() {
        Some(next) if next == "platform" => input.next(),
        _ => return Err(Error::Incomplete),
    };

    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async {
        match input.next() {
            Some(action) => match &action[..] {
                "build" => build(input, None).await.map(|_| ()),
                _ => Err(Error::BadInput([action, input.collect()].join(" "))),
            },
            _ => Err(Error::Incomplete),
        }
    })
}

#[test]
#[ignore]
fn test_build() {
    tokio_test::block_on(async {
        let empty = vec![String::new()];
        let mut input = empty.iter().cloned();
        let built = build(&mut input, Some("test")).await.unwrap();
        assert_eq!(built, "test-builder");
    });
}

async fn build<I: Iterator<Item = String>>(
    _input: &mut I,
    prefix: Option<&str>,
) -> Result<String, Error> {
    use bollard::image::BuildImageOptions;
    use bollard::Docker;
    use futures_util::stream::StreamExt;
    use std::io::{Read, Write};

    let context = dirs::home_dir().unwrap().join("local").join("jago");

    let mut template = String::new();
    let mut template_file = std::fs::File::open(context.join("container/builder/Dockerfile"))?;
    template_file.read_to_string(&mut template)?;

    let mut templates = Templates::new();
    templates.add_template("builder", &template)?;

    let library = library::inspect(&context)?;

    let rendered = templates.render("builder", &library.dependency_names())?;

    let path = context
        .join("container")
        .join("builder")
        .join("build.Dockerfile");

    let mut container_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)?;

    container_file.write_all(rendered.as_bytes())?;

    drop(container_file);

    let (key, done, server) = serve_build_context(&context, library)?;

    let build_context = get_build_context(&context)?;

    let docker = Docker::connect_with_unix_defaults()?;

    let tag = [prefix.unwrap_or(""), "builder"].join("-");

    let options = BuildImageOptions {
        dockerfile: path.to_str().unwrap(),
        t: &tag.clone(),
        remote: &key,
        ..Default::default()
    };

    let mut output = docker.build_image(options, None, Some(hyper::Body::from(build_context)));

    while let Some(msg) = output.next().await {
        println!("Message: {:?}", msg);
    }

    done();

    server.await?;

    Ok(tag)
}

#[test]
fn build_context() {
    use std::io::Write;

    use hyper::{body::HttpBody, Client};

    let context = dirs::home_dir().unwrap().join("local").join("jago");
    let library = library::inspect(&context).unwrap();

    let (definition, _) = tokio_test::block_on(async {
        let (key, stop, handle) =
            serve_build_context(&context, library.dependency_names()).unwrap();

        let definition = async {
            let client = Client::new();
            let uri = key.parse().unwrap();
            let mut resp = client.get(uri).await.unwrap();
            stop();

            let mut body = vec![];

            while let Some(chunk) = resp.body_mut().data().await {
                body.write_all(&chunk.unwrap()).unwrap();
            }

            String::from_utf8(body).unwrap()
        };

        futures::join!(definition, handle)
    });

    assert!(definition.starts_with("FROM"));
    assert!(!definition.contains("{{ endfor }}"));
    assert!(!definition.contains("html"));
}

fn serve_build_context<'a, S>(
    context: &'a Path,
    library: S,
) -> Result<(String, Box<dyn FnOnce()>, server::Handle<'a>), Error>
where
    S: Serialize,
{
    use futures::{channel::oneshot, future::FutureExt};

    let (stop, stopped) = oneshot::channel::<Result<(), Error>>();

    let handle = server::serve(async {
        let signalled = async { tokio::signal::ctrl_c().await.map_err(Error::from) };

        let _done = futures::select! {
            cancelled = signalled.fuse() => cancelled,
            stopped = stopped.fuse() => match stopped {
                Err(_cancelled) => {
                    log::info!("(kasdjhfl) sender cancelled");
                    Ok(())
                },
                _ => Ok(())
            },
        };
    });

    let library = serde_json::to_vec(&library)?;

    let key = format!(
        "http://0.0.0.0:1342/container/builder/Dockerfile?libraries={}",
        base64::encode(library)
    );

    let stop = move || {
        if let Err(_) = stop.send(Ok(())).map_err(|_| Error::Shutdown) {
            log::info!("(sfkhe) error sending stop signal");
        }
    };

    Ok((key, Box::new(stop), handle))
}

fn get_build_context(context: &Path) -> Result<Vec<u8>, Error> {
    let mut builder = ignore::WalkBuilder::new(&context);

    builder.add_ignore(context.join(".dockerignore"));
    builder.add_ignore(context.join(".gitignore"));

    let build_context = builder.build();

    let mut archive = tar::Builder::new(vec![]);

    for entry in build_context {
        let entry = entry?;
        let path = entry.path();

        if path == context {
            continue;
        }

        if let Some(kind) = entry.file_type() {
            let path = entry.path();
            let destination = path.strip_prefix(&context)?;

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
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Incomplete => write!(f, "incomplete"),
            Self::BadInput(input) => write!(f, "bad input: {}", input),
            Self::Template(error) => write!(f, "{}", error),
            Self::Machine(error) => write!(f, "{}", error),
            Self::Container(error) => write!(f, "{}", error),
            Self::Library(error) => write!(f, "{}", error),
            Self::Context(error) => write!(f, "{}", error),
            Self::Prefix(error) => write!(f, "{}", error),
            Self::Shutdown => write!(
                f,
                "Unknown error happened while shutting down server. You can probably ignore this."
            ),
            Self::Serve(error) => write!(f, "{}", error),
            Self::Serialize(error) => write!(f, "{}", error),
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

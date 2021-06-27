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

author::error!(
    Incomplete,
    BadInput(String),
    tinytemplate::error::Error,
    std::io::Error,
    bollard::errors::Error,
    library::Error,
    ignore::Error,
    std::path::StripPrefixError,
    (
        Shutdown,
        "Unknown error happened while shutting down server. You can probably ignore this."
    ),
    server::Error,
    serde_json::Error,
    shared::source::Error,
);

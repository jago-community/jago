use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use ignore::WalkBuilder;
use pulldown_cmark::{html, Options, Parser};

pub fn read<'a, W: Write>(target: &mut W, path: Arc<PathBuf>) -> Result<(), Error> {
    let metadata = std::fs::metadata(path.as_ref())?;

    if metadata.is_file() {
        read_file(target, path)?;
    } else {
        read_directory(target, path)?;
    }

    Ok(())
}

macro_rules! write_start {
    ( $( $target:expr, $context:expr )* ) => {
        write!(
            $(
                $target
            )*,
            "<!doctype html>\
            <html>\
                <head>\
                    <meta charset=\"utf-8\">\
                    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
                    <title>{context}</title>\

                    <style>
                        * {{
                            max-width: 100%;
                        }}
                    </style>
                </head>\
                <body>",
            context = $($context)*.unwrap_or("Jago")
        )
    };
}

macro_rules! write_end {
    ( $( $target:expr )* ) => {
        write!(
            $($target)*,
            "</body></html>"
        )
    };
}

macro_rules! write_document {
    ( $( $target:expr, $input:expr )* ) => {{
        let parser = Parser::new_ext($($input)*, Options::all());
        html::write_html($($target)*, parser)
    }};
}

pub fn read_directory<W: Write>(mut target: W, directory: Arc<PathBuf>) -> Result<(), Error> {
    write_start!(target, directory.file_name().and_then(|name| name.to_str()))?;

    let context = directory.file_name();

    let mut buffer = String::from("Directory:\n\n");

    let walker = WalkBuilder::new(directory.as_ref())
        .hidden(false)
        .max_depth(Some(1))
        .build();

    for entry in walker {
        let entry = entry?;

        let path = entry.path();

        if path == directory.as_ref() {
            continue;
        }

        if context == path.file_name() {
            read_file(&mut target, Arc::new(path.to_path_buf()))?;
        }

        let parent = match path.parent() {
            Some(parent) => parent,
            None => return Err(Error::NoParent(path.to_path_buf())),
        };

        let title = path.strip_prefix(parent)?;

        let context = dirs::home_dir().unwrap();

        let cleaned = path
            .strip_prefix(context.join("cache"))
            .unwrap_or(path.strip_prefix(context.join("local/jago"))?);

        buffer.push_str(&format!("- [{}]({})\n", title.display(), cleaned.display()));
    }

    write_document!(&mut target, &buffer)?;

    write_end!(target).map_err(Error::from)
}

fn read_file<W: Write>(target: &mut W, path: Arc<PathBuf>) -> Result<(), Error> {
    if crate::image::is_supported(path.as_ref()) {
        return read_image(target, path.as_ref());
    }

    read_document(target, path)
}

fn read_image<W: Write>(target: &mut W, path: &Path) -> Result<(), Error> {
    let mut buffer = vec![];

    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);

    reader.read_to_end(&mut buffer)?;

    target.write_all(&buffer).map_err(Error::from)
}

fn read_document<W: Write>(mut target: W, path: Arc<PathBuf>) -> Result<(), Error> {
    let file = std::fs::File::open(path.as_ref())?;

    write_start!(target, path.file_name().and_then(|name| name.to_str()))?;

    let mut reader = std::io::BufReader::new(file);

    let mut input = String::new();
    reader.read_to_string(&mut input)?;

    if let Some(extension) = path.extension() {
        if extension == "md" {
            write_document!(&mut target, &input)?;
        } else {
            write!(target, "<pre>{}</pre>", input)?;
        }
    } else {
        write_document!(&mut target, &input)?;
    }

    write_end!(target).map_err(Error::from)
}

#[derive(Debug)]
pub enum Error {
    Machine(std::io::Error),
    Write(std::io::IntoInnerError<std::io::BufWriter<Vec<u8>>>),
    NoParent(PathBuf),
    CleanPath(std::path::StripPrefixError),
    Walk(ignore::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Machine(error) => write!(f, "{}", error),
            Error::Write(error) => write!(f, "{}", error),
            Error::NoParent(path) => write!(f, "no parent for path: {}", path.display()),
            Error::CleanPath(error) => write!(f, "{}", error),
            Error::Walk(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Machine(error) => Some(error),
            Error::Write(error) => Some(error),
            Error::NoParent(_) => None,
            Error::CleanPath(error) => Some(error),
            Error::Walk(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Machine(error)
    }
}

impl From<std::io::IntoInnerError<std::io::BufWriter<Vec<u8>>>> for Error {
    fn from(error: std::io::IntoInnerError<std::io::BufWriter<Vec<u8>>>) -> Self {
        Self::Write(error)
    }
}

impl From<std::path::StripPrefixError> for Error {
    fn from(error: std::path::StripPrefixError) -> Self {
        Self::CleanPath(error)
    }
}

impl From<ignore::Error> for Error {
    fn from(error: ignore::Error) -> Self {
        Self::Walk(error)
    }
}

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

pub fn read_directory<W: Write>(target: &mut W, directory: Arc<PathBuf>) -> Result<(), Error> {
    let context = directory.file_name();

    let mut buffer = String::new();

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
            read_file(target, Arc::new(path.to_path_buf()))?;
        }

        let parent = match path.parent() {
            Some(parent) => parent,
            None => return Err(Error::NoParent(path.to_path_buf())),
        };

        let path = path.strip_prefix(parent)?;

        buffer.push_str(&format!("- [{path}]({path})\n", path = path.display()));
    }

    let parser = Parser::new_ext(&buffer, Options::all());
    html::write_html(target, parser)?;

    Ok(())
}

fn read_file<W: Write>(target: &mut W, path: Arc<PathBuf>) -> Result<(), Error> {
    if crate::image::is_supported(path.as_ref()) {
        return read_image(target, path.as_ref());
    }

    read_document(target, path)
}

fn read_image<W: Write>(target: &mut W, path: &Path) -> Result<(), Error> {
    let image = image::io::Reader::open(path)?;
    let format = match image.format() {
        Some(format) => format,
        None => return Err(Error::NoImageFormat(path.to_path_buf())),
    };
    let image = image.decode()?;

    image.write_to(target, format).map_err(Error::from)
}

fn read_document<W: Write>(mut target: W, path: Arc<PathBuf>) -> Result<(), Error> {
    let file = std::fs::File::open(path.as_ref())?;

    write!(
        target,
        "<!doctype html>\
        <html>\
            <head>\
                <meta charset=\"utf-8\">\
                <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
                <title>{context}</title>\
            </head>\
            <body>",
        context = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Jago")
    )?;

    let mut reader = std::io::BufReader::new(file);

    let mut input = String::new();
    reader.read_to_string(&mut input)?;

    if let Some(extension) = path.extension() {
        if extension == "md" {
            let parser = Parser::new_ext(&input, Options::all());
            html::write_html(&mut target, parser)?;
        } else {
            write!(target, "<pre>{}</pre>", input)?;
        }
    } else {
        let parser = Parser::new_ext(&input, Options::all());
        html::write_html(&mut target, parser)?;
    }

    write!(target, "</body></html>").map_err(Error::from)
}

#[derive(Debug)]
pub enum Error {
    Machine(std::io::Error),
    Write(std::io::IntoInnerError<std::io::BufWriter<Vec<u8>>>),
    Image(image::error::ImageError),
    NoImageFormat(PathBuf),
    NoParent(PathBuf),
    CleanPath(std::path::StripPrefixError),
    Walk(ignore::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Machine(error) => write!(f, "{}", error),
            Error::Write(error) => write!(f, "{}", error),
            Error::Image(error) => write!(f, "{}", error),
            Error::NoImageFormat(path) => write!(f, "no image format for: {}", path.display()),
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
            Error::Image(error) => Some(error),
            Error::NoImageFormat(_) => None,
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

impl From<image::error::ImageError> for Error {
    fn from(error: image::error::ImageError) -> Self {
        Self::Image(error)
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

use std::{
    collections::HashMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use ignore::WalkBuilder;
use pulldown_cmark::{html, Options, Parser};
use tinytemplate::TinyTemplate as Templates;

pub type Variables<'a> = HashMap<&'a str, serde_json::Value>;

pub fn read<'a, W: Write>(
    target: &mut W,
    path: Arc<PathBuf>,
    variables: Option<Variables<'a>>,
) -> Result<(), Error> {
    let metadata = std::fs::metadata(path.as_ref())?;

    let variables = variables.unwrap_or_else(|| HashMap::new());

    if metadata.is_file() {
        read_file(target, path, &variables)?;
    } else {
        read_directory(target, path, &variables)?;
    }

    Ok(())
}

macro_rules! write_start {
    ( $( $target:expr, $context:expr, $style:expr )* ) => {{
        write!(
            $(
                $target
            )*,
            "<!doctype html>\
            <html>\
                <head>\
                    <meta charset=\"utf-8\">\
                    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
                    <title>{context}</title>",
            context = $($context)*.unwrap_or("Jago")
        )?;

        if let Some(path) = $($style)* {
            let context = dirs::home_dir().unwrap();

            let cleaned = match path.strip_prefix(context.join("cache")) {
                Ok(cleaned) => cleaned,
                _ => match path.strip_prefix(context.join("local/jago")) {
                    Ok(cleaned) => cleaned,
                    Err(error) => return Err(Error::from(error)),
                },
            };

            write!($($target)*, "<link rel=\"stylesheet\" href=\"{}\">", cleaned.display())?;
        }


        write!(
            $($target)*,
            "</head><body>"
        )
    }};
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

pub fn read_directory<'a, W: Write>(
    mut target: W,
    directory: Arc<PathBuf>,
    variables: &'a Variables<'a>,
) -> Result<(), Error> {
    let maybe_style_path = {
        let mut style_path = directory.as_ref().to_path_buf();
        style_path.set_extension("css");

        if std::fs::metadata(&style_path).is_ok() {
            Some(style_path)
        } else {
            None
        }
    };

    write_start!(
        target,
        directory.file_name().and_then(|name| name.to_str()),
        maybe_style_path
    )?;

    let context = directory.file_stem();

    let mut buffer = String::from(
        "<details>\n\
        <summary>Directory:</summary>\n\n",
    );

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
            read_file(&mut target, Arc::new(path.to_path_buf()), variables)?;
        }

        let parent = match path.parent() {
            Some(parent) => parent,
            None => return Err(Error::NoParent(path.to_path_buf())),
        };

        let title = path.strip_prefix(parent)?;

        let context = dirs::home_dir().unwrap();

        let cleaned = match path.strip_prefix(context.join("cache")) {
            Ok(cleaned) => cleaned,
            _ => match path.strip_prefix(context.join("local/jago")) {
                Ok(cleaned) => cleaned,
                Err(error) => return Err(Error::from(error)),
            },
        };

        buffer.push_str(&format!("- [{}]({})\n", title.display(), cleaned.display()));
    }

    buffer.push_str("</details>");

    write_document!(&mut target, &buffer)?;

    write_end!(target).map_err(Error::from)
}

fn read_file<'a, W: Write>(
    target: &mut W,
    path: Arc<PathBuf>,
    variables: &'a Variables<'a>,
) -> Result<(), Error> {
    let is_style = path
        .extension()
        .map(|extension| extension == "css")
        .unwrap_or(false);

    if is_style || crate::image::is_supported(path.as_ref()) {
        return read_content(target, path.as_ref());
    }

    let is_container_definition = path
        .file_stem()
        .map(|stem| stem == "Dockerfile")
        .unwrap_or(false);

    if is_container_definition {
        return read_template(target, path.as_ref(), variables);
    }

    read_document(target, path)
}

fn read_content<'a, W: Write>(target: &mut W, path: &Path) -> Result<(), Error> {
    let mut buffer = vec![];

    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);

    reader.read_to_end(&mut buffer)?;

    target.write_all(&buffer).map_err(Error::from)
}

fn read_template<'a, W: Write>(
    target: &mut W,
    path: &Path,
    variables: &'a Variables<'a>,
) -> Result<(), Error> {
    let mut buffer = vec![];

    read_content(&mut buffer, path)?;

    let template = String::from_utf8(buffer)?;

    let mut templates = Templates::new();
    let key = path.display().to_string();

    templates.add_template(&key, &template)?;

    let rendered = templates.render(&key, variables)?;

    log::info!("{}", rendered);

    target.write_all(rendered.as_bytes()).map_err(Error::from)
}

fn read_document<W: Write>(mut target: W, path: Arc<PathBuf>) -> Result<(), Error> {
    let file = std::fs::File::open(path.as_ref())?;

    let maybe_style_path = {
        let mut style_path = path.as_ref().to_path_buf();
        style_path.set_extension("css");

        if std::fs::metadata(&style_path).is_ok() {
            Some(style_path)
        } else {
            None
        }
    };

    write_start!(
        target,
        path.file_name().and_then(|name| name.to_str()),
        maybe_style_path
    )?;

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
    FromUtf8(std::string::FromUtf8Error),
    Template(tinytemplate::error::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Machine(error) => write!(f, "{}", error),
            Error::Write(error) => write!(f, "{}", error),
            Error::NoParent(path) => write!(f, "no parent for path: {}", path.display()),
            Error::CleanPath(error) => write!(f, "{}", error),
            Error::Walk(error) => write!(f, "{}", error),
            Error::FromUtf8(error) => write!(f, "{}", error),
            Error::Template(error) => write!(f, "{}", error),
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
            Error::FromUtf8(error) => Some(error),
            Error::Template(error) => Some(error),
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

impl From<std::string::FromUtf8Error> for Error {
    fn from(error: std::string::FromUtf8Error) -> Self {
        Self::FromUtf8(error)
    }
}

impl From<ignore::Error> for Error {
    fn from(error: ignore::Error) -> Self {
        Self::Walk(error)
    }
}

impl From<tinytemplate::error::Error> for Error {
    fn from(error: tinytemplate::error::Error) -> Self {
        Self::Template(error)
    }
}

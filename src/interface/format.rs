use std::io::Write;

pub fn content<'a, W: Write>(writer: &mut W, input: &'a str) -> Result<(), Error> {
    use pulldown_cmark::{Event, Tag};

    let parser = pulldown_cmark::Parser::new_ext(input, pulldown_cmark::Options::all());

    pulldown_cmark::html::write_html(writer, parser)?;

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Machine(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Machine(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Machine(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Machine(error)
    }
}

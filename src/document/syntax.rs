use std::borrow::Cow;

use std::io::{Read, Write};

pub fn write<'a, R: Read, W: Write>(mut reader: R, writer: &mut W) -> Result<(), Error> {
    use pulldown_cmark::{html, Parser};

    let mut input = String::new();
    reader.read_to_string(&mut input)?;

    let parser = Parser::new(&input);

    html::write_html(writer, parser)?;

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Machine(std::io::Error),
    Parse(ParseError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Machine(error) => write!(f, "{}", error),
            Error::Parse(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Machine(error) => Some(error),
            Error::Parse(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Machine(error)
    }
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Self {
        Self::Parse(error)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParseError {
    input: String,
    kind: ParseErrorKind,
    backtrace: Vec<ParseError>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseErrorKind {
    Parse(nom::error::ErrorKind),
    Incomplete(nom::Needed),
}

impl nom::error::ParseError<&str> for ParseError {
    fn from_error_kind(input: &str, kind: nom::error::ErrorKind) -> Self {
        ParseError {
            input: input.into(),
            kind: ParseErrorKind::Parse(kind),
            backtrace: vec![],
        }
    }

    fn append(input: &str, kind: nom::error::ErrorKind, mut other: Self) -> Self {
        other.backtrace.push(Self::from_error_kind(input, kind));
        other
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ParseErrorKind::Incomplete(needed) => write!(
                f,
                "incomplete data ({}) input = {}",
                match needed {
                    nom::Needed::Unknown => "unknown".into(),
                    nom::Needed::Size(size) => format!("missing {}", size),
                },
                self.input
            ),
            ParseErrorKind::Parse(kind) => {
                write!(f, "{} - input = {}", kind.description(), self.input)
            }
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

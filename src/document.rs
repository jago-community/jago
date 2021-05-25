mod parse;
mod syntax;
mod write;

use std::{
    borrow::Cow,
    io::{Read, Write},
};

#[derive(Clone, Debug, PartialEq)]
pub enum Expression<'a> {
    Break,
    String(Cow<'a, str>),
    Link(Cow<'a, str>, Cow<'a, str>),
    Combination(Vec<Expression<'a>>),
}

pub fn html<'a, R: Read, W: Write>(
    reader: R,
    writer: &mut W,
    context: Option<&'a str>,
) -> Result<(), Error> {
    write!(
        writer,
        "<!doctype html>\
        <html>\
            <head>\
                <title>{context}</title>\
            </head>\
            <body>",
        context = context.unwrap_or("Jago")
    )?;
    syntax::write(reader, writer)?;
    write!(writer, "</body></html>",)?;
    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Parse(parse::Error),
    Write(write::Error),
    Syntax(syntax::Error),
    /* Rage */ Machine(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(error) => write!(f, "{}", error),
            Error::Write(error) => write!(f, "{}", error),
            Error::Syntax(error) => write!(f, "{}", error),
            Error::Machine(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Write(error) => Some(error),
            Error::Parse(error) => Some(error),
            Error::Syntax(error) => Some(error),
            Error::/* Rage */Machine(error) => Some(error),
        }
    }
}

impl From<syntax::Error> for Error {
    fn from(error: syntax::Error) -> Self {
        Self::Syntax(error)
    }
}

impl From<parse::Error> for Error {
    fn from(error: parse::Error) -> Self {
        Self::Parse(error)
    }
}

impl From<write::Error> for Error {
    fn from(error: write::Error) -> Self {
        Self::Write(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        // Rage
        Self::Machine(error)
    }
}

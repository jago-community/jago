use std::io::Write;

use crate::parse::Document;

#[test]
fn test_html() {
    let input = include_str!("../jago");

    let want = "\
        <p>Intro, Jago</p>\
        <p>Of random company</p>\
        <p>Pertinent:</p>\
        <p>\
            <a href=\"terms-of-service\">Terms of service.</a>\
            <a href=\"privacy-policy\">Privacy Policy.</a>\
        </p>\
        <p>Other:</p>\
        <p>\
            <a href=\"random-kind\">Random kind.</a>\
        </p>\
    ";
}

/// ```doc
/// Intro, Jago
///
/// Of random company
///
/// Pertinent:
///
/// [!Aa.](terms-of-service)
/// [!Aa.](privacy-policy)
///
/// Other:
///
/// [!Aa.](random-kind)
/// ```
pub fn html<'a, W: Write>(w: &mut W, tree: Document<'a>) -> Result<(), Error> {
    if let Some(stem) = tree.stem {
        kind(w, &stem)?;
    }

    Ok(())
}

use crate::parse::Kind;

pub fn kind<'a, W: Write>(writer: &mut W, this: &'a Kind<'a>) -> Result<(), Error> {
    match &this {
        Kind::Link(text, destination) => {
            write!(writer, "<a href=\"")?;
            kind(writer, destination.as_ref())?;
            write!(writer, ">{}</a>", text)?;
        }
        Kind::Rest(text) => {
            writer.write(&text.as_bytes())?;
        }
        _ => {
            panic!("{:?}", this);
        }
    };
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
        Self::Machine(error)
    }
}

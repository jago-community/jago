use std::borrow::Cow;
use std::io::Write;

#[test]
#[ignore]
fn test_translate() {
    let mut output = vec![];
    translate(&mut output, include_str!("../../jago")).unwrap();
    assert_eq!(
        std::str::from_utf8(&output).unwrap(),
        "<p>The following may or may not be interesting to you.</p>\
        <p><a href=\"./start\">./start</a></p>"
    );
}

pub fn translate<'a, W: Write>(writer: &mut W, input: &'a str) -> Result<(), Error> {
    let tree = parse(input)?;

    if let Some(stem) = tree.stem {
        write_stem(writer, &stem)?;
    }

    Ok(())
}

pub fn write_stem<'a, W: Write>(writer: &mut W, stem: &'a Stem<'a>) -> Result<(), Error> {
    println!("---> {:?}", stem);
    match stem {
        Stem::NewLine => {
            writer.write(&[b'\n'])?;
        }
        Stem::Expression(parts) => {
            let mut group = vec![];
            for stem in parts {
                if let &Stem::Text(_) = stem.as_ref() {
                    group.push(stem);
                } else if let &Stem::Link { .. } = stem.as_ref() {
                    group.push(stem);
                } else if &Stem::NewLine == stem.as_ref() {
                    if group.len() > 0 {
                        write!(writer, "<p>")?;
                        for stem in &group {
                            write_stem(writer, stem)?;
                        }
                        write!(writer, "</p>")?;
                    }
                } else {
                    write_stem(writer, stem.as_ref())?;
                }
            }
        }
        Stem::Text(text) => {
            write!(writer, "{}", text)?;
        }
        Stem::Link {
            destination,
            representation: _,
            description: _,
        } => {
            write!(
                writer,
                "<a href=\"{destination}\">{destination}</a>",
                destination = destination,
            )?;
        }
    };

    Ok(())
}

#[test]
fn test_parse() {
    assert_eq!(
        parse(include_str!("../../jago")).unwrap().stem,
        Some(Stem::Expression(vec![
            Box::new(Stem::Text(
                "The following may or may not be interesting to you.".into()
            )),
            Box::new(Stem::NewLine),
            Box::new(Stem::Link {
                destination: "./start".into(),
                representation: None,
                description: None,
            }),
            Box::new(Stem::NewLine),
        ]))
    );
}

pub fn parse<'a>(input: &'a str) -> Result<Tree<'a>, Error> {
    let (_, tree) = tree(input).map_err(|error: nom::Err<ParseError>| {
        Error::Parse(match error {
            nom::Err::Incomplete(needed) => ParseError {
                input: input.into(),
                kind: ParseErrorKind::Incomplete(needed),
                backtrace: vec![],
            },
            nom::Err::Error(error) | nom::Err::Failure(error) => ParseError {
                input: input.into(),
                kind: error.kind,
                backtrace: vec![],
            },
        })
    })?;

    Ok(tree)
}

pub fn tree<'a>(input: &'a str) -> nom::IResult<&'a str, Tree<'a>, ParseError> {
    use nom::{
        branch::alt,
        bytes::complete::{is_not, tag, take_till},
        combinator::map,
        multi::fold_many0,
        sequence::{delimited, pair},
    };

    fold_many0(
        alt((
            map(
                pair(
                    delimited(tag("["), is_not("]"), tag("]")),
                    delimited(tag("("), is_not(")"), tag(")")),
                ),
                |(text, destination): (&str, &str)| {
                    vec![Box::new(Stem::Link(text.into(), destination.into()))]
                },
            ),
            map(
                pair(take_till(|c: char| c == '\n'), tag("\n")),
                |(text, _): (&str, _)| {
                    if text == "" {
                        vec![]
                    } else {
                        vec![Box::new(Stem::Text(text.into())), Box::new(Stem::NewLine)]
                    }
                },
            ),
        )),
        Tree::default(),
        |mut tree, stems| {
            tree.stem = tree
                .stem
                .map(|previous| match previous {
                    Stem::Expression(mut before) => {
                        for stem in &stems {
                            before.push(stem.to_owned());
                        }
                        Stem::Expression(before)
                    }
                    before @ _ => {
                        let mut before = vec![Box::new(before)];
                        for stem in &stems {
                            before.push(stem.to_owned());
                        }
                        Stem::Expression(before)
                    }
                })
                .or(Some(Stem::Expression(stems)));

            tree
        },
    )(input)
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Tree<'a> {
    stem: Option<Stem<'a>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stem<'a> {
    Text(Cow<'a, str>),
    NewLine,
    Link(Cow<'a, str>, Cow<'a, str>),
    Expression(Vec<Box<Stem<'a>>>),
}

impl Default for Stem<'_> {
    fn default() -> Self {
        Self::Expression(vec![])
    }
}

#[derive(Debug)]
pub enum Error {
    Parse(ParseError),
    Machine(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(error) => write!(f, "{}", error),
            Error::Machine(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Parse(error) => Some(error),
            Error::Machine(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Machine(error)
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

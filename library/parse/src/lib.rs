mod protected;

pub fn with_fn<'a, T>(
    content: &'a [u8],
    parse: fn(&'a [u8]) -> nom::IResult<&'a [u8], T, Error>,
) -> Result<T, Error> {
    let (_, parsed) = parse(content).map_err(|error: nom::Err<Error>| match error {
        nom::Err::Error(error) | nom::Err::Failure(error) => error,
        nom::Err::Incomplete(needed) => Error {
            input: content.into(),
            kind: ErrorKind::Incomplete(needed),
            backtrace: vec![],
        },
    })?;

    Ok(parsed)
}

use either::Either;

#[derive(Clone)]
pub struct Output<I: ToOwned> {
    buffers: Vec<Either<I, I::Owned>>,
}

impl<I: ToOwned> Output<I> {
    fn new() -> Self {
        Self { buffers: vec![] }
    }

    fn push(&mut self, item: I) {
        self.buffers.push(Either::Left(item));
    }

    fn push_owned(&mut self, item: I::Owned) {
        self.buffers.push(Either::Right(item));
    }
}

#[test]
fn test_markup() {
    use std::str;

    let input = b"# Hello, reader";
    let want = b"<h1>Hello, reader</h1>";
    let (_, got) = markup(input).unwrap();

    assert_eq!(str::from_utf8(want.as_ref()), str::from_utf8(&got));
}

use std::io::Write;

pub fn markup<'a>(input: &'a [u8]) -> nom::IResult<&'a [u8], Vec<u8>, Error> {
    use nom::{
        branch::alt,
        bytes::complete::{tag, take, take_till},
        character::is_newline,
        combinator::map,
        multi::fold_many0,
        sequence::{preceded, terminated},
    };

    unimplemented!()

    /*
    fold_many0(
        alt((
            map(
                preceded(tag("# "), terminated(take_till(is_newline), take(1))),
                |(level, block)| (1, block),
            ),
            map(
                preceded(tag("## "), terminated(take_till(is_newline), take(1))),
                |(level, block)| (2, block),
            ),
        )),
        vec![],
        |mut output, (level, block): (u8, &[u8])| {
            write!(
                &mut output,
                "<h{level}>{block}</h{level}>",
                level = level,
                block = match std::str::from_utf8(block) {
                    Ok(block) => block,
                    Err(_) => "<binary>",
                },
            )
            .unwrap();

            output
        },
    )(input)*/
}

#[derive(Debug, PartialEq, Clone)]
pub struct Error {
    input: Vec<u8>,
    kind: ErrorKind,
    backtrace: Vec<Error>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ErrorKind {
    Parse(nom::error::ErrorKind),
    Incomplete(nom::Needed),
    Syntax(String),
}

impl nom::error::ParseError<&[u8]> for Error {
    fn from_error_kind(input: &[u8], kind: nom::error::ErrorKind) -> Self {
        Error {
            input: input.into(),
            kind: ErrorKind::Parse(kind),
            backtrace: vec![],
        }
    }

    fn append(input: &[u8], kind: nom::error::ErrorKind, mut other: Self) -> Self {
        other.backtrace.push(Self::from_error_kind(input, kind));
        other
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ErrorKind::Incomplete(needed) => write!(
                f,
                "incomplete data ({}) input = {}",
                match needed {
                    nom::Needed::Unknown => "unknown".into(),
                    nom::Needed::Size(size) => format!("missing {}", size),
                },
                match std::str::from_utf8(&self.input) {
                    Ok(input) => input.into(),
                    Err(_) => format!("{:?}", self.input),
                }
            ),
            ErrorKind::Parse(kind) => {
                write!(
                    f,
                    "{} - input = {}",
                    kind.description(),
                    match std::str::from_utf8(&self.input) {
                        Ok(input) => input.into(),
                        Err(_) => format!("{:?}", self.input),
                    }
                )
            }
            ErrorKind::Syntax(error) => {
                write!(f, "{}", error)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

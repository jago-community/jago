use crate::input::{Input, WrappedInput};

pub fn search<'a>(input: &'a str, pattern: &'a str) -> Result<(), Error> {
    with_fn(input.as_bytes(), matched(pattern))
}

use nom::error::ParseError;

#[test]
fn test_matched() {
    let tests = vec![
        (b"Hello, world!".as_ref(), "Hello,", Ok(())),
        (b"Hello, world!", "wo", Ok(())),
        (
            b"Hello, world!",
            "war",
            Err(nom::Err::Error(Error::from_error_kind(
                Input::from(&b"Hello, world!"[..]),
                nom::error::ErrorKind::TakeUntil,
            ))),
        ),
    ];

    for (input, pattern, want) in tests {
        let got = matched(dbg!(pattern))(Input::from(input));

        assert_eq!(got.is_ok(), want.is_ok());
    }
}

use nom::{
    bytes::complete::{tag, take_until},
    combinator::value,
    sequence::pair,
    IResult,
};

fn matched<'a>(pattern: &'a str) -> impl Fn(Input<'a>) -> IResult<Input<'a>, (), Error> {
    move |input: Input<'a>| value((), pair(take_until(pattern), tag(pattern)))(input)
}

/*
#[test]
fn test_all_matches() {
    let tests = vec![
        (
            "Hello, world!",
            "l",
            Ok((
                "",
                vec![
                    (Input::from(&b"lo, world!"[..]), ("He".len(), "l".len())),
                    ("o, world!".into(), ("Hel".len(), "l".len())),
                    ("d!".into(), ("Hello, wor".len(), "l".len())),
                ],
            )),
        ),
        (
            "Hello, world!",
            "war",
            Err(nom::Err::Error(nom::error::Error::from_error_kind(
                LocatedSpan::new("Hello, world!"),
                nom::error::ErrorKind::Many1,
            ))),
        ),
    ];

    for (input, pattern, want) in tests {
        let got = many1_matched_positions(pattern)(input.into());

        let want_succeed = want.is_ok();

        assert_eq!(got.is_ok(), want_succeed);

        if want_succeed {
            let want = want.unwrap().1;

            for (at, got) in got.unwrap().1.iter().enumerate() {
                assert_eq!(got, &want[at].1);
            }
        }
    }
}

use nom::multi::many1;

pub fn many1_matched_positions<'a>(
    pattern: &'a str,
) -> impl Fn(LocatedSpan<&'a str>) -> IResult<LocatedSpan<&'a str>, Vec<(usize, usize)>, Error> {
    move |input: LocatedSpan<&'a str>| many1(matched_position(pattern))(input)
}
*/

pub fn with_fn<'a, T>(
    input: &'a [u8],
    parse: impl Fn(Input<'a>) -> nom::IResult<Input<'a>, T, Error>,
) -> Result<T, Error> {
    let (_, parsed) = parse(Input::from(input)).map_err(|error: nom::Err<Error>| match error {
        nom::Err::Error(error) | nom::Err::Failure(error) => error,
        nom::Err::Incomplete(needed) => Error {
            input: Input::from(input.as_ref()).take_all().wrap(),
            kind: ErrorKind::Incomplete(needed),
            backtrace: vec![],
        },
    })?;

    Ok(parsed)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Error {
    input: WrappedInput,
    kind: ErrorKind,
    backtrace: Vec<Error>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ErrorKind {
    Parse(nom::error::ErrorKind),
    Incomplete(nom::Needed),
    Syntax(String),
}

use nom::InputTake;

impl nom::error::FromExternalError<&str, Error> for Error {
    fn from_external_error(input: &str, kind: nom::error::ErrorKind, error: Error) -> Self {
        Self {
            input: Input::from(input.as_bytes()).take_all().wrap(),
            kind: ErrorKind::Parse(kind),
            backtrace: vec![error],
        }
    }
}

impl ParseError<&str> for Error {
    fn from_error_kind(input: &str, kind: nom::error::ErrorKind) -> Self {
        Self {
            input: Input::from(input.as_bytes()).take(input.len()).wrap(),
            kind: ErrorKind::Parse(kind),
            backtrace: vec![],
        }
    }

    fn append(input: &str, kind: nom::error::ErrorKind, mut other: Self) -> Self {
        other.backtrace.push(Self::from_error_kind(input, kind));
        other
    }
}

impl<'a> nom::error::FromExternalError<Input<'a>, Error> for Error {
    fn from_external_error(input: Input<'a>, kind: nom::error::ErrorKind, error: Error) -> Self {
        Self {
            input: input.wrap(),
            kind: ErrorKind::Parse(kind),
            backtrace: vec![error],
        }
    }
}

impl<'a> nom::error::ParseError<Input<'a>> for Error {
    fn from_error_kind(input: Input<'a>, kind: nom::error::ErrorKind) -> Self {
        Self {
            input: input.wrap(),
            kind: ErrorKind::Parse(kind),
            backtrace: vec![],
        }
    }

    fn append(input: Input<'a>, kind: nom::error::ErrorKind, mut other: Self) -> Self {
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
                self.input
            ),
            ErrorKind::Parse(kind) => {
                write!(f, "{} - input = `{}`", kind.description(), self.input)
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

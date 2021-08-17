#[test]
fn test_matched() {
    let tests = vec![
        ("Hello, world!", "Hello,", Ok((" world!", ()))),
        ("Hello, world!", "wo", Ok(("rld!".into(), ()))),
        (
            "Hello, world!",
            "war",
            Err(nom::Err::Error(nom::error::Error::from_error_kind(
                "Hello, world!",
                nom::error::ErrorKind::TakeUntil,
            ))),
        ),
    ];

    for (input, pattern, want) in tests {
        let got = matched(pattern)(input.into());

        let want_succeed = want.is_ok();

        assert_eq!(got.is_ok(), want_succeed);

        if want_succeed {
            assert_eq!(got.unwrap().1, want.unwrap().1);
        }
    }
}

use nom::{
    bytes::complete::{tag, take_until},
    combinator::value,
    error::ParseError,
    IResult,
};

pub fn matched<'a>(pattern: &'a str) -> impl Fn(&'a str) -> IResult<&'a str, (), Error> {
    move |input: &'a str| {
        let (input, _) = take_until(pattern)(input)?;
        value((), tag(pattern))(input)
    }
}

#[test]
fn test_matched_count() {
    let tests = vec![
        ("Hello, world!", "l", Ok(("", 3))),
        (
            "Hello, world!",
            "war",
            Err(nom::Err::Error(nom::error::Error::from_error_kind(
                "Hello, world!",
                nom::error::ErrorKind::Many1,
            ))),
        ),
    ];

    for (input, pattern, want) in tests {
        let got = matched_count(pattern)(input.into());

        let want_succeed = want.is_ok();

        assert_eq!(got.is_ok(), want_succeed);

        if want_succeed {
            let want = want.unwrap().1;

            assert_eq!(got.unwrap().1, want);
        }
    }
}

use nom::multi::{many1, many1_count};

pub fn matched_count<'a>(pattern: &'a str) -> impl Fn(&'a str) -> IResult<&'a str, usize, Error> {
    move |input: &'a str| many1_count(matched(pattern))(input)
}

pub fn with_fn<'a, T>(
    input: &'a str,
    parse: impl Fn(&'a str) -> nom::IResult<&'a str, T, Error>,
) -> Result<T, Error> {
    use nom::ExtendInto;

    let (_, parsed) = parse(input.into()).map_err(|error: nom::Err<Error>| match error {
        nom::Err::Error(error) | nom::Err::Failure(error) => error,
        nom::Err::Incomplete(needed) => Error {
            input: {
                let mut output = String::new();
                input.extend_into(&mut output);
                output
            },
            kind: ErrorKind::Incomplete(needed),
            backtrace: vec![],
        },
    })?;

    Ok(parsed)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Error {
    input: String,
    kind: ErrorKind,
    backtrace: Vec<Error>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ErrorKind {
    Parse(nom::error::ErrorKind),
    Incomplete(nom::Needed),
    Syntax(String),
}

impl<'a> nom::error::FromExternalError<&'a str, Error> for Error {
    fn from_external_error(input: &'a str, kind: nom::error::ErrorKind, error: Error) -> Self {
        Self {
            input: input.into(),
            kind: ErrorKind::Parse(kind),
            backtrace: vec![error],
        }
    }
}

impl<'a> nom::error::ParseError<&'a str> for Error {
    fn from_error_kind(input: &'a str, kind: nom::error::ErrorKind) -> Self {
        Self {
            input: input.into(),
            kind: ErrorKind::Parse(kind),
            backtrace: vec![],
        }
    }

    fn append(input: &'a str, kind: nom::error::ErrorKind, mut other: Self) -> Self {
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
                write!(f, "{} - input = {}", kind.description(), self.input)
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

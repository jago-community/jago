use nom::error::ParseError;

#[test]
fn test_matches() {
    let tests = vec![
        (
            "Hello, world!",
            "Hello,",
            Ok((LocatedSpan::new(" world!"), (0, "Hello,".len()))),
        ),
        (
            "Hello, world!",
            "wo",
            Ok(("rld!".into(), ("Hello, ".len(), "wo".len()))),
        ),
        (
            "Hello, world!",
            "war",
            Err(nom::Err::Error(nom::error::Error::from_error_kind(
                LocatedSpan::new("Hello, world!"),
                nom::error::ErrorKind::TakeUntil,
            ))),
        ),
    ];

    for (input, pattern, want) in tests {
        let got = matched_position(pattern)(input.into());

        let want_succeed = want.is_ok();

        assert_eq!(got.is_ok(), want_succeed);

        if want_succeed {
            assert_eq!(got.unwrap().1, want.unwrap().1);
        }
    }
}

use nom::{
    bytes::complete::{tag, take_until},
    IResult,
};

use nom_locate::{position, LocatedSpan};

fn matched_position<'a>(
    pattern: &'a str,
) -> impl Fn(LocatedSpan<&'a str>) -> IResult<LocatedSpan<&'a str>, (usize, usize)> {
    move |input: LocatedSpan<&'a str>| {
        let (input, _) = take_until(pattern)(input)?;
        let (input, position) = position(input)?;
        let (input, matched) = tag(pattern)(input)?;

        Ok((input, (position.location_offset(), matched.len())))
    }
}

#[test]
fn test_all_matches() {
    let tests = vec![
        (
            "Hello, world!",
            "l",
            Ok((
                "",
                vec![
                    (LocatedSpan::new("lo, world!"), ("He".len(), "l".len())),
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
        let got = all_matched_positions(pattern)(input.into());

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

fn all_matched_positions<'a>(
    pattern: &'a str,
) -> impl Fn(LocatedSpan<&'a str>) -> IResult<LocatedSpan<&'a str>, Vec<(usize, usize)>> {
    move |input: LocatedSpan<&'a str>| many1(matched_position(pattern))(input)
}

pub fn with_fn<'a, T>(
    content: &'a str,
    parse: impl Fn(&'a str) -> nom::IResult<&'a str, T, Error>,
) -> Result<T, Error> {
    let (_, parsed) = parse(content).map_err(|error: nom::Err<Error>| match error {
        nom::Err::Error(error) | nom::Err::Failure(error) => error,
        nom::Err::Incomplete(needed) => Error {
            input: content.to_string(),
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

impl nom::error::FromExternalError<&str, Error> for Error {
    fn from_external_error(input: &str, kind: nom::error::ErrorKind, error: Error) -> Self {
        Self {
            input: input.into(),
            kind: ErrorKind::Parse(kind),
            backtrace: vec![error],
        }
    }
}

impl ParseError<&str> for Error {
    fn from_error_kind(input: &str, kind: nom::error::ErrorKind) -> Self {
        Self {
            input: input.into(),
            kind: ErrorKind::Parse(kind),
            backtrace: vec![],
        }
    }

    fn append(input: &str, kind: nom::error::ErrorKind, mut other: Self) -> Self {
        other.backtrace.push(Self::from_error_kind(input, kind));
        other
    }
}

impl<'a> nom::error::FromExternalError<LocatedSpan<&'a str>, Error> for Error {
    fn from_external_error(
        input: LocatedSpan<&'a str>,
        kind: nom::error::ErrorKind,
        error: Error,
    ) -> Self {
        Self {
            input: input.to_string(),
            kind: ErrorKind::Parse(kind),
            backtrace: vec![error],
        }
    }
}

impl<'a> nom::error::ParseError<LocatedSpan<&'a str>> for Error {
    fn from_error_kind(input: LocatedSpan<&'a str>, kind: nom::error::ErrorKind) -> Self {
        Self {
            input: input.to_string(),
            kind: ErrorKind::Parse(kind),
            backtrace: vec![],
        }
    }

    fn append(input: LocatedSpan<&'a str>, kind: nom::error::ErrorKind, mut other: Self) -> Self {
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

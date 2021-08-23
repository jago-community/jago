use nom::IResult;

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
        let pattern = regex::RegexBuilder::new(pattern)
            .case_insensitive(true)
            .build()
            .unwrap();

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
    regexp::str::re_find,
};

use regex::Regex;

pub fn matched<'a>(pattern: Regex) -> impl Fn(&'a str) -> IResult<&'a str, (), Error> {
    move |input: &'a str| value((), re_find(pattern.clone()))(input)
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
        let pattern = regex::RegexBuilder::new(pattern)
            .case_insensitive(true)
            .build()
            .unwrap();

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

pub fn matched_count<'a>(pattern: Regex) -> impl Fn(&'a str) -> IResult<&'a str, usize, Error> {
    move |input: &'a str| many1_count(matched(pattern.clone()))(input)
}

#[test]
fn test_tagged_lines() {
    let input = "
        hello world
        haha
        what the
        sitemap: alkjseflk;ajsfljajoasd
        woop there it was
        another one!
        sitemap: l;kasjdfafsdlj
        see that?
        ";

    let want = vec!["alkjseflk;ajsfljajoasd", "l;kasjdfafsdlj"];

    let pattern = regex::RegexBuilder::new("sitemap:")
        .case_insensitive(true)
        .build()
        .unwrap();

    let got = tagged_lines(pattern)(input).unwrap();

    assert_eq!(got.1, want);
}

use nom::{
    bytes::complete::take_till,
    character::complete::space0,
    combinator::map,
    multi::many0,
    sequence::{terminated, tuple},
};

pub fn tagged_lines<'a>(
    pattern: Regex,
) -> impl Fn(&'a str) -> IResult<&'a str, Vec<&'a str>, Error> {
    move |input: &'a str| {
        many0(map(
            tuple((matched(pattern.clone()), space0, line_content)),
            |(_, _, line)| line,
        ))(input)
    }
}

#[test]
#[ignore]
fn test_line_content() {
    let input = "hello
world
yo
        ";

    let want = vec!["hello", "world", "yo"];
    let got = many0(line_content)(input).unwrap();

    assert_eq!(got.1, want);
}

use nom::character::{
    complete::{multispace1, newline, not_line_ending},
    is_newline,
};

fn line_content<'a>(input: &'a str) -> IResult<&'a str, &'a str, Error> {
    terminated(not_line_ending, multispace1)(input)
}

pub fn with_fn<'a, Output>(
    input: &'a str,
    combinator: impl Fn(&'a str) -> IResult<&'a str, Output, Error>,
) -> Result<Output, Error> {
    use nom::ExtendInto;

    let (_, parsed) = combinator(input.into()).map_err(|error: nom::Err<Error>| match error {
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
    Regex(regex::Error),
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
            ErrorKind::Regex(error) => {
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

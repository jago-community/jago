type Input<'a> = &'a str;
type Output = String;

use std::collections::HashMap;

#[derive(Default, Clone)]
struct Context<'a> {
    definitions: HashMap<Input<'a>, Input<'a>>,
    buffer: Output,
}

#[test]
fn test_expand() {
    let tests = vec![
        (
            "cargo test {--package:library/shell/src/lib.rs}",
            "cargo test --package shell",
        ),
        ("cargo test --workspace", "cargo test --workspace"),
    ];

    for (input, want) in tests {
        let got = expand(input).unwrap();

        assert_eq!(want, dbg!(&got[..]));
    }
}

pub fn expand<'a>(input: Input<'a>) -> Result<Output, Error> {
    use nom::{branch::alt, combinator::map_res, multi::fold_many0, sequence::pair};

    let parse = |i| {
        map_res(
            fold_many0(
                pair(
                    take_until("{"),
                    alt((
                        map_res(keyed_value("--package"), |value| {
                            context::package(Some(value))
                                .map(|value| ("--package", value))
                                .map_err(|error| Error {
                                    input: input.into(),
                                    kind: ErrorKind::Context(error),
                                    backtrace: vec![],
                                })
                        }),
                        map(key_value, |(key, value)| (key, value.into())),
                    )),
                ),
                (None, Output::new()),
                |(previous, mut output): (Option<(&str, String)>, String),
                 (before, (key, value))| {
                    output.push_str(before);

                    (
                        if let Some((previous_key, previous_value)) = previous {
                            if value == previous_key {
                                output.push_str(key);
                                output.push(' ');
                                output.push_str(previous_value.as_ref());
                            }

                            None
                        } else {
                            Some((key, value))
                        },
                        output,
                    )
                },
            ),
            |(rest, output)| match rest {
                Some((key, value)) => Ok(format!("{}{} {}", dbg!(output), dbg!(key), value)),
                None => Ok(output),
            },
        )(i)
    };

    with_fn(parse, input)
}

#[test]
fn test_key_value() {
    let tests = vec![
        (
            "{%:library/shell/src/lib.rs}",
            ("%", "library/shell/src/lib.rs"),
        ),
        ("{package:%}", ("package", "%")),
    ];

    for (input, want) in tests {
        let got = with_fn(key_value, input).unwrap();

        assert_eq!(want, got);
    }
}

use nom::{
    bytes::complete::{tag, take_until},
    combinator::map,
    sequence::tuple,
};

fn keyed_value<'a>(
    key: &'a str,
) -> impl Fn(Input<'a>) -> nom::IResult<Input<'a>, Input<'a>, Error> {
    move |input: Input<'a>| {
        map(
            tuple((tag("{"), tag(key), tag(":"), take_until("}"), tag("}"))),
            |(_, _, _, value, _)| value,
        )(input)
    }
}

fn key_value<'a>(input: Input<'a>) -> nom::IResult<Input<'a>, (Input<'a>, Input<'a>), Error> {
    map(
        tuple((
            tag("{"),
            take_until(":"),
            tag(":"),
            take_until("}"),
            tag("}"),
        )),
        |(_, key, _, value, _)| (key, value),
    )(input)
}

pub fn with_fn<'a, T>(
    parse: impl Fn(Input<'a>) -> nom::IResult<Input<'a>, T, Error>,
    input: Input<'a>,
) -> Result<T, Error> {
    let (_, parsed) = parse(input).map_err(|error: nom::Err<Error>| match error {
        nom::Err::Error(error) | nom::Err::Failure(error) => error,
        nom::Err::Incomplete(needed) => Error {
            input: input.into(),
            kind: ErrorKind::Incomplete(needed),
            backtrace: vec![],
        },
    })?;

    Ok(parsed)
}

#[derive(Debug)]
pub struct Error {
    input: Output,
    kind: ErrorKind,
    backtrace: Vec<Error>,
}

#[derive(Debug)]
pub enum ErrorKind {
    Parse(nom::error::ErrorKind),
    Incomplete(nom::Needed),
    Syntax(String),
    Context(context::Error),
}

use nom::InputTake;

impl<'a> nom::error::FromExternalError<Input<'a>, Error> for Error {
    fn from_external_error(input: Input<'a>, kind: nom::error::ErrorKind, error: Error) -> Self {
        Self {
            input: input.into(),
            kind: ErrorKind::Parse(kind),
            backtrace: vec![error],
        }
    }
}

impl<'a> nom::error::ParseError<Input<'a>> for Error {
    fn from_error_kind(input: Input<'a>, kind: nom::error::ErrorKind) -> Self {
        Self {
            input: input.into(),
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
                write!(f, "syntax: {}", error)
            }
            ErrorKind::Context(error) => {
                write!(f, "context: {}", error)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

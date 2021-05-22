use std::borrow::Cow;

use super::Expression;

pub fn unwrapped<'a>(input: &'a str) -> Result<Expression<'a>, Error> {
    let (_, output) = expression(input).map_err(|error: nom::Err<Error>| match error {
        nom::Err::Error(error) | nom::Err::Failure(error) => error,
        nom::Err::Incomplete(needed) => Error {
            input: input.into(),
            kind: ErrorKind::Incomplete(needed),
            backtrace: vec![],
        },
    })?;

    Ok(output)
}

/*
Intro, Jago

Of random company

Pertinent:

[Terms of service.](..)
[Privacy policy.](..)

Other:

[Random.](..kind)
 */
#[test]
fn test_expression() {
    let cases = vec![
        ("Intro, Jago", Expression::String("Intro, Jago".into())),
        ("[a](link)", Expression::Link("a".into(), "link".into())),
        (
            "[Terms of service.](%)",
            Expression::Link("Terms of service.".into(), "terms-of-service".into()),
        ),
        (
            "[Random.](%-kind)",
            Expression::Link("Random.".into(), "random-kind".into()),
        ),
        (
            "Intro, Jago

Of random company",
            Expression::Combination(vec![
                Expression::String("Intro, Jago".into()),
                Expression::Break,
                Expression::Break,
                Expression::String("Of random company".into()),
            ]),
        ),
        (
            "Intro, Jago

Of random company

Pertinent:

[Terms of service.](%)
[Privacy policy.](%)

Other:

[Random.](%-kind)",
            Expression::Combination(vec![
                Expression::String("Intro, Jago".into()),
                Expression::Break,
                Expression::Break,
                Expression::String("Of random company".into()),
                Expression::Break,
                Expression::Break,
                Expression::String("Pertinent:".into()),
                Expression::Break,
                Expression::Break,
                Expression::Link("Terms of service.".into(), "terms-of-service".into()),
                Expression::Break,
                Expression::Link("Privacy policy.".into(), "privacy-policy".into()),
                Expression::Break,
                Expression::Break,
                Expression::String("Other:".into()),
                Expression::Break,
                Expression::Break,
                Expression::Link("Random.".into(), "random-kind".into()),
            ]),
        ),
    ];

    for (input, want) in cases {
        let (_, got) = expression(input).unwrap();
        assert_eq!(got, want);
    }
}

use nom::{
    branch::alt,
    character::complete::{line_ending, not_line_ending},
    combinator::map,
    multi::many0,
};

fn expression<'a>(input: &'a str) -> nom::IResult<&'a str, Expression<'a>, Error> {
    let (input, mut expressions) = many0(single)(input)?;

    let expression = match expressions.len() {
        1 => expressions.pop().unwrap(),
        _ => Expression::Combination(expressions),
    };

    Ok((input, expression))
}

fn single<'a>(input: &'a str) -> nom::IResult<&'a str, Expression<'a>, Error> {
    if input.is_empty() {
        return Err(nom::Err::Error(Error {
            input: input.into(),
            kind: ErrorKind::Incomplete(nom::Needed::new(1)),
            backtrace: vec![],
        }));
    }

    alt((
        map(line_ending, |_| Expression::Break),
        map(link, |(text, destination)| {
            Expression::Link(text, destination)
        }),
        map(not_line_ending, |text: &str| {
            Expression::String(text.into())
        }),
    ))(input)
}

#[test]
fn test_link() {
    let cases = vec![
        ("[a](link)", ("a".into(), "link".into())),
        (
            "[Terms of service.](%)",
            ("Terms of service.".into(), "terms-of-service".into()),
        ),
        (
            "[Random.](%-kind)",
            ("Random.".into(), "random-kind".into()),
        ),
        (
            "[Random.org](bounty/%)",
            ("Random.org".into(), "bounty/random.org".into()),
        ),
    ];

    for (input, want) in cases {
        let (_, got) = link(input).unwrap();
        assert_eq!(got, want);
    }
}

use nom::{
    bytes::complete::{is_not, tag},
    sequence::{delimited, pair},
};

fn link<'a>(input: &'a str) -> nom::IResult<&'a str, (Cow<'a, str>, Cow<'a, str>), Error> {
    let (input, (text, destination)) = pair(
        delimited(tag("["), is_not("]"), tag("]")),
        delimited(tag("("), is_not(")"), tag(")")),
    )(input)?;

    // TODO: make expand optional so it doesn't fail here.
    //
    let destination = expand(destination, text).map_err(|error| nom::Err::Failure(error))?;

    Ok((input, (text.into(), destination.into())))
}

fn expand<'a>(input: &'a str, context: &'a str) -> Result<Cow<'a, str>, Error> {
    use unicode_segmentation::UnicodeSegmentation;

    let words = context
        .unicode_words()
        .map(|word| word.to_lowercase())
        .collect::<Vec<_>>()
        .join("-");

    let output = input.replace("%", &words);

    Ok(output.into())
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
}

impl nom::error::ParseError<&str> for Error {
    fn from_error_kind(input: &str, kind: nom::error::ErrorKind) -> Self {
        Error {
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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ErrorKind::Incomplete(needed) => write!(
                f,
                "incomplete data ({}) input = {}",
                match needed {
                    nom::Needed::Unknown => "unknown".into(),
                    nom::Needed::Size(size) => format!("missing {}", size),
                },
                self.input
            ),
            ErrorKind::Parse(kind) => write!(f, "{} - input = {}", kind.description(), self.input),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

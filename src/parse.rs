use std::borrow::Cow;

pub fn parse<'a>(content: &'a str) -> Result<Document<'a>, Error> {
    let (_, parsed) = document(content).map_err(|error: nom::Err<Error>| match error {
        nom::Err::Error(error) | nom::Err::Failure(error) => error,
        nom::Err::Incomplete(needed) => Error {
            input: content.into(),
            kind: ErrorKind::Incomplete(needed),
            backtrace: vec![],
        },
    })?;

    Ok(parsed)
}

#[derive(Debug, PartialEq)]
pub struct Document<'a> {
    pub stem: Option<Kind<'a>>,
}

fn doc<'a>(kind: Kind<'a>) -> Box<Document<'a>> {
    Box::new(Document { stem: Some(kind) })
}

#[derive(Debug, PartialEq)]
pub enum Kind<'a> {
    Break,
    Text(Cow<'a, str>),
    Rest(Cow<'a, str>),
    Expression(Vec<Kind<'a>>),
    Link(Cow<'a, str>, Box<Kind<'a>>),
    Guard(Box<Kind<'a>>),
    Random(Box<Kind<'a>>),
}

#[test]
#[ignore]
fn test_document() {
    let input = include_str!("../jago");

    let want = Document {
        stem: Some(Kind::Expression(vec![
            Kind::Text("Intro, Jago".into()),
            Kind::Break,
            Kind::Text("Of random company".into()),
            Kind::Break,
            Kind::Text("Pertinent:".into()),
            Kind::Break,
            Kind::Link(
                "Terms of service.".into(),
                Box::new(Kind::Rest("terms-of-service".into())),
            ),
            Kind::Break,
            Kind::Link(
                "Privacy policy.".into(),
                Box::new(Kind::Rest("privacy-policy".into())),
            ),
            Kind::Break,
            Kind::Text("Other:".into()),
            Kind::Break,
            Kind::Link(
                "Random.".into(),
                Box::new(Kind::Expression(vec![
                    Kind::Guard(Box::new(Kind::Rest("agreement".into()))),
                    Kind::Random(Box::new(Kind::Rest("kind".into()))),
                ])),
            ),
        ])),
    };

    let (_, got) = document(input).unwrap();

    assert_eq!(want, got);
}

use nom::{combinator::map, multi::many0};

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
/// [!Aa.](!random-kind)
/// ```
fn document<'a>(input: &'a str) -> nom::IResult<&'a str, Document<'a>, Error> {
    if input == "" {
        return Err(nom::Err::Error(Error {
            input: input.into(),
            kind: ErrorKind::Incomplete(nom::Needed::Unknown),
            backtrace: vec![],
        }));
    }

    map(many0(kind), |kind| Document {
        stem: Some(Kind::Expression({
            let mut stem = vec![];
            for kind in kind {
                if let Some(kind) = kind {
                    stem.push(kind);
                }
            }
            stem
        })),
    })(input)
}

#[test]
#[ignore]
fn test_kind() {
    let input = include_str!("../jago").split('\n');

    let mut wants = vec![
        Kind::Rest("Intro, Jago".into()),
        Kind::Rest("Of random company".into()),
        Kind::Rest("Pertinent:".into()),
        Kind::Link(
            "Terms of service.".into(),
            Box::new(Kind::Rest("terms-of-service".into())),
        ),
        Kind::Link(
            "Privacy policy.".into(),
            Box::new(Kind::Rest("privacy-policy".into())),
        ),
        Kind::Rest("Other:".into()),
        Kind::Link(
            "Random kind.".into(),
            Box::new(Kind::Rest("random-kind".into())),
        ),
    ];

    wants.reverse();

    for line in input {
        let (_, got) = kind(line).unwrap();
        if let Some(got) = got {
            assert_eq!(&got, wants.last().unwrap());
            wants.pop();
        }
    }

    assert_eq!(wants.len(), 0)
}

use nom::{
    branch::alt,
    bytes::complete::is_not,
    character::complete::{line_ending, not_line_ending},
    sequence::pair,
};

fn kind<'a>(input: &'a str) -> nom::IResult<&'a str, Option<Kind<'a>>, Error> {
    if input == "" {
        return Ok((input, None));
    }

    alt((
        map(line_ending, |_| Some(Kind::Break)),
        map(link, |(text, destination)| {
            Some(Kind::Link(text, Box::new(destination)))
        }),
        map(not_line_ending, |rest: &str| Some(Kind::Rest(rest.into()))),
    ))(input)
}

#[test]
#[ignore]
fn test_link() {
    let cases = vec![
        (
            "[!Aa.](terms-of-service)",
            (
                "Terms of service.".into(),
                Kind::Rest("terms-of-service".into()),
            ),
        ),
        (
            "[!Aa.](random-kind)",
            ("Random kind.".into(), Kind::Rest("random-kind".into())),
        ),
    ];

    for (input, want) in cases {
        let (_, got) = link(input).unwrap();
        assert_eq!(got, want);
    }
}

use nom::{bytes::complete::tag, sequence::delimited};

fn link<'a>(input: &'a str) -> nom::IResult<&'a str, (Cow<'a, str>, Kind<'a>), Error> {
    let (input, (text, destination)) = pair(
        delimited(tag("["), is_not("]"), tag("]")),
        delimited(tag("("), is_not(")"), tag(")")),
    )(input)?;

    let (_, text) = expand(text, destination)?;

    let (_, destination_kind) = kind(destination)?;

    Ok((
        input,
        (
            text,
            if destination_kind.is_some() {
                destination_kind.unwrap()
            } else {
                Kind::Rest(destination.into())
            },
        ),
    ))
}

#[test]
fn test_expand() {
    let cases = vec![("!Aa.", "terms-of-service", "Terms of service.")];

    for (input, context, want) in cases {
        let (_, got) = expand(input, context).unwrap();
        assert_eq!(got, want);
    }
}

use nom::combinator::value;

fn expand<'a>(input: &'a str, context: &'a str) -> nom::IResult<&'a str, Cow<'a, str>, Error> {
    let mut output = context.into();

    let (input, is_expression) = value(true, pair(tag("!"), tag("Aa.")))(input)?;

    if is_expression {
        output = sentence(context);
    }

    Ok((input, output))
}

#[test]
fn test_sentence() {
    assert_eq!(sentence("a-b-c"), Cow::from("A b c."));
}

fn sentence<'a>(input: &'a str) -> Cow<'a, str> {
    let mut output = String::new();

    let mut characters = input.chars();

    while let Some(first) = characters.next() {
        if first.is_alphabetic() {
            let idealized = first.to_uppercase().to_string();
            output.push_str(&idealized);
            break;
        }
    }

    let mut output =
        characters
            .map(|c| if c == '-' { ' ' } else { c })
            .fold(output, |mut output, character| {
                output.push(character);
                output
            });

    output.push('.');

    output.into()
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

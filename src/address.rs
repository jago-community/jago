/// Good enough address parsing capability.
pub fn parse<'a>(content: &'a str) -> Result<Address<'a>, Error> {
    let (_, parsed) = address(content).map_err(|error: nom::Err<Error>| match error {
        nom::Err::Error(error) | nom::Err::Failure(error) => error,
        nom::Err::Incomplete(needed) => Error {
            input: content.into(),
            kind: ErrorKind::Incomplete(needed),
            backtrace: vec![],
        },
    })?;

    Ok(parsed)
}

#[derive(Debug, PartialEq, Eq)]
pub struct Address<'a> {
    source: &'a str,
    parent: &'a str,
    name: &'a str,
    path: Option<&'a str>,
}

use std::path::PathBuf;

impl<'a> Address<'a> {
    pub fn join_context(&self, context: PathBuf) -> PathBuf {
        let path = context.join(self.source);

        if let Some(rest) = self.path {
            path.join(rest)
        } else {
            path
        }
    }

    pub fn source(&self) -> &'a str {
        self.source
    }
}

fn address<'a>(i: &'a str) -> nom::IResult<&'a str, Address<'a>, Error> {
    nom::combinator::map(
        nom::sequence::pair(source, nom::combinator::rest),
        |((source, parent, name), path)| Address {
            source,
            parent,
            name,
            path: if path == "" { None } else { Some(&path[1..]) },
        },
    )(i)
}

#[test]
#[ignore]
fn test_address() {
    let cases = vec![
        (
            "git@github.com:jago-community/jago.git/usage",
            Address {
                source: "git@github.com:jago-community/jago.git",
                parent: "jago-community",
                name: "jago",
                path: Some("usage"),
            },
        ),
        (
            "/start",
            Address {
                source: "git@github.com:jago-community/jago.git",
                parent: "jago-community",
                name: "jago",
                path: Some("usage"),
            },
        ),
    ];
    for (input, want) in cases {
        assert_eq!(address(input).unwrap(), ("", want));
    }
}

fn source<'a>(i: &'a str) -> nom::IResult<&'a str, (&'a str, &'a str, &'a str), Error> {
    let parser = |x| nom::sequence::tuple((actor, host, segment_hard, segment_soft))(x);

    let (i, source) = nom::combinator::recognize(parser)(i)?;

    let (_, (_, _, parent, name)) = parser(source)?;

    Ok((
        i,
        (source, parent, name.strip_suffix(".git").unwrap_or(name)),
    ))
}

#[test]
fn test_source() {
    assert_eq!(
        source("git@github.com:jago-community/jago.git/usage").unwrap(),
        (
            "/usage",
            (
                "git@github.com:jago-community/jago.git",
                "jago-community",
                "jago"
            )
        )
    );
}

fn actor<'a>(i: &'a str) -> nom::IResult<&'a str, (&'a str, Option<&'a str>), Error> {
    nom::sequence::terminated(
        nom::sequence::separated_pair(
            nom::character::complete::alphanumeric1,
            nom::combinator::opt(nom::bytes::complete::tag(":")),
            nom::combinator::opt(nom::character::complete::alphanumeric1),
        ),
        nom::bytes::complete::tag("@"),
    )(i)
}

#[test]
fn test_actor() {
    assert_eq!(actor("git@rest").unwrap(), ("rest", ("git", None)));
    assert_eq!(
        actor("git:pass@rest").unwrap(),
        ("rest", ("git", Some("pass")))
    );
}

fn host<'a>(i: &'a str) -> nom::IResult<&'a str, &'a str, Error> {
    nom::sequence::terminated(
        nom::bytes::complete::take_while(|c: char| c.is_alphanumeric() || c == '-' || c == '.'),
        nom::combinator::opt(nom::bytes::complete::take(1usize)),
    )(i)
}

#[test]
fn test_host() {
    assert_eq!(host("github.com").unwrap(), ("", "github.com"));
    assert_eq!(host("localhost/test").unwrap(), ("test", "localhost"));
}

fn segment_hard<'a>(i: &'a str) -> nom::IResult<&'a str, &'a str, Error> {
    nom::sequence::terminated(
        nom::bytes::complete::take_while(|c: char| c != '/'),
        nom::combinator::opt(nom::bytes::complete::take(1usize)),
    )(i)
}

fn segment_soft<'a>(i: &'a str) -> nom::IResult<&'a str, &'a str, Error> {
    nom::bytes::complete::take_while(|c: char| c != '/')(i)
}

#[test]
fn test_segment() {
    assert_eq!(
        segment_hard("some-space/name/path").unwrap(),
        ("name/path", "some-space")
    );
    assert_eq!(
        segment_hard("localhost/test").unwrap(),
        ("test", "localhost")
    );
    assert_eq!(
        segment_soft("some-space/name/path").unwrap(),
        ("/name/path", "some-space")
    );
    assert_eq!(
        segment_soft("localhost/test").unwrap(),
        ("/test", "localhost")
    );
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

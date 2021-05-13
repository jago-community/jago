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
    path: Option<&'a str>,
}

fn address<'a>(i: &'a str) -> nom::IResult<&'a str, Address<'a>, Error> {
    nom::combinator::map(
        nom::sequence::pair(source, nom::combinator::rest),
        |(source, path)| Address {
            source,
            path: if path == "" { None } else { Some(&path[1..]) },
        },
    )(i)
}

#[test]
fn test_address() {
    assert_eq!(
        address("git@github.com:jago-contributors/jago.git/usage").unwrap(),
        (
            "",
            Address {
                source: "git@github.com:jago-contributors/jago.git",
                path: Some("usage"),
            }
        )
    );
}

fn source<'a>(i: &'a str) -> nom::IResult<&'a str, &'a str, Error> {
    nom::combinator::recognize(nom::sequence::tuple((
        actor,
        host,
        segment_hard,
        segment_soft,
    )))(i)
}

#[test]
fn test_source() {
    assert_eq!(
        source("git@github.com:jago-community/jago.git/usage").unwrap(),
        ("/usage", "git@github.com:jago-community/jago.git")
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

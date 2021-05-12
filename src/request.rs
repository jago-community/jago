use std::borrow::Cow;

pub fn parse<'a>(input: &'a str) -> Result<Request<'a>, Error> {
    let (_, parsed) = request(input).map_err(|error: nom::Err<Error>| match error {
        nom::Err::Error(error) | nom::Err::Failure(error) => error,
        nom::Err::Incomplete(needed) => Error {
            input: input.into(),
            kind: ErrorKind::Incomplete(needed),
            backtrace: vec![],
        },
    })?;

    Ok(parsed)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Request<'a> {
    Check(Option<Box<Request<'a>>>),
    Reference(Cow<'a, str>),
}

fn request<'a>(input: &'a str) -> nom::IResult<&'a str, Request<'a>, Error> {
    /*
    nom::branch::alt((
            nom::combinator::value(Action::Check, nom::bytes::complete::tag("check")),
            nom::combinator::map(
                nom::bytes::complete::take_till(|c: char| c == ' '),
                |path: &str| Action::Path(path.into()),
            ),
            nom::combinator::value(Action::Path("/".into()), nom::bytes::complete::tag("/")),
        ))*/
    nom::combinator::value(Request::Check(None), nom::bytes::complete::tag("check"))(input)
}

#[test]
fn test_request() {
    let cases = vec![
        ("check", Request::Check(None)),
        (
            "check git@github.com:vim/vim.git",
            Request::Check(Some(Box::new(Request::Reference(
                "git@github.com:vim/vim.git".into(),
            )))),
        ),
        (
            "git@github.com:jago-contributors/jago.git",
            Request::Reference("git@github.com:jago-community/jago.git".into()),
        ),
    ];

    for (arguments, want) in cases {
        let (_, got) = request(arguments).unwrap();

        assert_eq!(got, want);
    }
}

#[derive(Debug, PartialEq)]
pub struct Error {
    input: String,
    kind: ErrorKind,
    backtrace: Vec<Error>,
}

#[derive(Debug, PartialEq)]
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

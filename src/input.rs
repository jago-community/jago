use std::borrow::Cow;

pub fn parse<'a>(content: &'a str) -> Result<Input<'a>, Error> {
    let (_, parsed) = input(content).map_err(|error: nom::Err<Error>| match error {
        nom::Err::Error(error) | nom::Err::Failure(error) => error,
        nom::Err::Incomplete(needed) => Error {
            input: content.into(),
            kind: ErrorKind::Incomplete(needed),
            backtrace: vec![],
        },
    })?;

    Ok(parsed)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Input<'a> {
    Check(Option<Box<Input<'a>>>),
    Serve(Option<Box<Input<'a>>>),
    Prepare,
    Rest(Cow<'a, str>),
}

impl Default for Input<'_> {
    fn default() -> Self {
        Self::Check(None)
    }
}

fn input<'a>(content: &'a str) -> nom::IResult<&'a str, Input<'a>, Error> {
    if content == "" {
        return Err(nom::Err::Error(Error {
            input: content.into(),
            kind: ErrorKind::Incomplete(nom::Needed::Unknown),
            backtrace: vec![],
        }));
    }

    nom::branch::alt((
        nom::combinator::map(
            nom::sequence::separated_pair(
                nom::bytes::complete::tag("serve"),
                nom::character::complete::space0,
                nom::combinator::opt(input),
            ),
            |(_, request): (_, Option<Input<'a>>)| {
                Input::Serve(request.map(|request| Box::new(request)))
            },
        ),
        nom::combinator::map(
            nom::sequence::separated_pair(
                nom::bytes::complete::tag("check"),
                nom::character::complete::space0,
                nom::combinator::opt(input),
            ),
            |(_, request): (_, Option<Input<'a>>)| {
                Input::Check(request.map(|request| Box::new(request)))
            },
        ),
        nom::combinator::map(nom::bytes::complete::tag("prepare"), |_| Input::Prepare),
        nom::combinator::map(nom::combinator::rest, |rest: &str| Input::Rest(rest.into())),
    ))(content)
}

#[test]
fn test_request() {
    let cases = vec![
        ("check", Input::Check(None)),
        ("serve", Input::Serve(None)),
        (
            "check git@github.com:vim/vim.git",
            Input::Check(Some(Box::new(Input::Rest(
                "git@github.com:vim/vim.git".into(),
            )))),
        ),
        (
            "git@github.com:jago-community/jago.git",
            Input::Rest("git@github.com:jago-community/jago.git".into()),
        ),
    ];

    for (arguments, want) in cases {
        let (_, got) = input(arguments).unwrap();

        assert_eq!(got, want);
    }
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

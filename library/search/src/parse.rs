use nom::{
    bytes::complete::tag,
    character::complete::{alphanumeric1, line_ending},
    combinator::map_res,
    multi::many0,
    sequence::terminated,
    IResult,
};

pub fn end_of_line(input: &str) -> IResult<&str, &str, Error> {
    if input.is_empty() {
        Ok((input, input))
    } else {
        line_ending(input)
    }
}

pub fn line(input: &str) -> IResult<&str, &str, Error> {
    terminated(alphanumeric1, end_of_line)(input)
}

pub fn sitemap(input: &str) -> IResult<&str, Vec<&str>, Error> {
    many0(map_res(line, |line| with_fn::<&str>(line, tag("sitemap:"))))(input)
}

pub fn with_fn<'a, T>(
    content: &'a str,
    parse: fn(&'a str) -> nom::IResult<&'a str, T, Error>,
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

impl nom::error::ParseError<&str> for Error {
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

use nom::IResult;

#[test]
#[ignore]
fn test_markup() {
    let cases = vec![
        (
            "Right now<#000>,<#> we<#fff>'<#>re one<a random color>.<a> But not for long<a random color>.<a>",
            "Right now<span style=\"color:#000\">,</span> we<span style=\"color:#fff\">'</span>re one<span style=\"color:#0079FF\">.</span> But not for long<span style=\"color:#0079FF\">.</span>",
        )
    ];

    for (input, want) in cases {
        let (_, got) = markup(input).unwrap();
        assert_eq!(got, want);
    }
}

fn markup(input: &str) -> IResult<&str, &str, Error> {
    unimplemented!()
}

#[test]
fn test_angle() {
    let cases = vec![
        ("<#000>,<#000>", "<span style=\"color:#000\">,</span>"),
        (
            "<a random color>.<a random color>",
            "<span style=\"color:#0079FF\">.</span>",
        ),
        ("<#000>,<#>", "<span style=\"color:#000\">,</span>"),
        (
            "<a random color>.<a>",
            "<span style=\"color:#0079FF\">.</span>",
        ),
    ];

    for (input, want) in cases {
        let (_, got) = angle(input).unwrap();
        assert_eq!(&got, want);
    }
}

use nom::{
    bytes::complete::{tag, take_till},
    sequence::delimited,
};

fn angle(input: &str) -> IResult<&str, String, Error> {
    let (input, start) = delimited(tag("<"), take_till(|c| c == '>'), tag(">"))(input)?;
    let (input, until_tag) = take_till(|c| c == '<')(input)?;
    let (input, end) = delimited(tag("<"), tag(start), tag(">"))(input)?;
    let (input, end) = delimited(tag("<"), tag(start), tag(">"))(input)?;

    Ok((input, format!("{} - {} - {}", start, until_tag, end)))
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
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

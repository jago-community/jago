pub fn populate() -> Result<(), Error> {
    local()
}

#[test]
fn test_populate() {
    populate().unwrap();
    assert!(std::env::var("IDENTITY").ok() != Some("".into()));
}

fn local() -> Result<(), Error> {
    use std::io::Read;

    let local_file = dirs::home_dir().unwrap().join("local/jago/local");

    let environment_file = match std::fs::metadata(&local_file) {
        Ok(_) => std::fs::File::open(local_file)?,
        Err(_) => return Err(Error::Missing(Paths(vec!["local".into(), local_file]))),
    };

    let mut reader = std::io::BufReader::new(&environment_file);

    let mut raw_environment = String::new();

    reader.read_to_string(&mut raw_environment)?;

    let variables = environment(&raw_environment)?;

    for (key, value) in variables {
        let expanded = shellexpand::full(value)?;
        let expanded = expanded.to_string();
        std::env::set_var(key, &expanded);
    }

    Ok(())
}

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, line_ending, not_line_ending},
    combinator::map,
    multi::many0,
    sequence::separated_pair,
    IResult,
};

pub fn environment(i: &str) -> Result<Vec<(&str, &str)>, Error> {
    let (_, variables) = many0(alt((map(variable, Some), map(comment, |_| None))))(i).map_err(
        |error: nom::Err<parse::Error>| {
            Error::ParseError(match error {
                nom::Err::Incomplete(needed) => parse::Error {
                    input: i.into(),
                    kind: parse::ErrorKind::Incomplete(needed),
                    backtrace: vec![],
                },
                nom::Err::Error(error) | nom::Err::Failure(error) => parse::Error {
                    input: i.into(),
                    kind: error.kind,
                    backtrace: vec![],
                },
            })
        },
    )?;

    Ok(variables.iter().filter_map(|tuple| *tuple).collect())
}

#[test]
fn test_environment() {
    let raw = include_str!("../../../local");
    let list = environment(raw).unwrap();

    assert!(list.len() > 0);

    let mut sane = false;

    for (key, value) in list {
        if key == "IDENTITY" {
            assert!(value != "~/.ssh/id_rsa");
            sane = true;
        }
    }

    assert!(sane);
}

use nom::sequence::pair;

pub fn variable<'a>(i: &'a str) -> IResult<&'a str, (&'a str, &'a str), parse::Error> {
    map(
        pair(
            separated_pair(alpha1, tag("="), not_line_ending),
            line_ending,
        ),
        |(key_value, _)| key_value,
    )(i)
}

use nom::{combinator::value, sequence::tuple};

pub fn comment<'a>(i: &'a str) -> IResult<&'a str, (), parse::Error> {
    value((), tuple((tag("#"), not_line_ending, tag("\n"))))(i)
}

author::error!(
    std::io::Error,
    parse::Error,
    Missing(Paths),
    shellexpand::LookupError<std::env::VarError>,
);

#[derive(Debug)]
pub struct Paths(Vec<std::path::PathBuf>);

impl std::fmt::Display for Paths {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "paths: {}",
            self.0
                .iter()
                .map(|key| key.to_str().unwrap_or("<bad path name>"))
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

mod parse {
    #[derive(Debug, PartialEq, Clone)]
    pub struct Error {
        pub input: String,
        pub kind: ErrorKind,
        pub backtrace: Vec<Error>,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub enum ErrorKind {
        Parse(nom::error::ErrorKind),
        Incomplete(nom::Needed),
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
}

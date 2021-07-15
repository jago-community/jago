// - create library with specific dependencies
// - walks specified root directory and
//   - runs macros on files
//   - creates tmp directory with processed file contents
//   - rest of input is command to run in created directory

author::error!(Incomplete, parse::Error);

#[test]
fn test_process_input() {
    let tests = vec![(
        "cargo add {self} crate",
        "cargo add --manifest-path=library/shell/Cargo.toml crate",
    )];

    for (input, want) in tests {
        let mut input = input.split(" ").map(String::from);
        assert_eq!(want, process_input(&mut input).unwrap());
    }
}

pub fn process_input<I: Iterator<Item = String>>(input: &mut I) -> Result<String, Error> {
    let input = input.fold(String::new(), |mut input, next| {
        input.push_str(&next);
        input
    });

    unimplemented!()

    /*
    parse::with_fn(&input, parse::input)
        .map_err(Error::from)
        .map(|segments| {
            segments.iter().map(|segment| match segment {
                parse::Segment::Part(part) => Ok(part),
                parse::Segment::CurrentLocation => unimplemented!(),
            })
        })
    */
}

mod parse {
    #[derive(Clone)]
    pub enum Segment<'a> {
        Part(&'a str),
        CurrentLocation,
    }

    use nom::{
        branch::alt,
        bytes::complete::{tag, take_till},
        combinator::{map, value},
        multi::many0,
        sequence::delimited,
    };

    pub fn input<'a>(input: &'a str) -> nom::IResult<&'a str, Vec<Segment<'a>>, Error> {
        many0(alt((
            map(take_till(|c| c == ' '), |part| Segment::Part(part)),
            value(
                Segment::CurrentLocation,
                delimited(tag("{"), tag("self"), tag("}")),
            ),
        )))(input)
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

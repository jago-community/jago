// - create library with specific dependencies
// - walks specified root directory and
//   - runs macros on files
//   - creates tmp directory with processed file contents
//   - rest of input is command to run in created directory

author::error!(Incomplete, expand::Error);

#[test]
#[ignore]
fn test_expand() {
    let tests = vec![
        //("cargo add {self} crate", "")
        //("cargo add --manifest-path=library/shell/Cargo.toml crate", "")
        (
            r#"

        jago! [
               "logger",
               "server",
               "storage",
               "platform",
               "interface",
               "lense",
               "window",
               "filter",
               "shell"
           ] => "{} = {{ path = "library/{}", optional = true }}"



        "#,
            r#"logger = { path = "library/logger", optional = true }
server = { path = "library/server", optional = true }
storage = { path = "library/storage", optional = true }
platform = { path = "library/platform", optional = true }
interface = { path = "library/interface", optional = true }
lense = { path = "library/lense", optional = true }
#window = { path = "library/window", optional = true }
filter = { path = "library/filter", optional = true }
shell = { path = "library/shell", optional = true }"#,
        ),
    ];

    for (input, want) in tests {
        let mut input = input.split(" ").map(String::from);
        assert_eq!(want, expand_iterator(&mut input).unwrap());
    }
}

pub fn expand_iterator<I: Iterator<Item = String>>(input: &mut I) -> Result<String, Error> {
    let input = input.fold(String::new(), |mut input, next| {
        input.push_str(&next);
        input
    });

    expand::with_fn(&input, expand::str).map_err(Error::from)
}

mod expand {
    #[test]
    fn test_str() {
        let input = "rust!(library::libraries()?
    .map(|library| format!(
            \"{0} = {{ path = \"library/{0}\", optional = true }}\", library)
        )



        );";

        let (_, block) = str(input).unwrap();
        assert_eq!(
            block,
            "library::libraries()?
    .map(|library| format!(
            \"{} = {{ path = \"library/{}\", optional = true }}\")
        )



    "
        );
    }

    use quote::quote;

    use nom::{
        bytes::complete::{tag, take_till},
        combinator::recognize,
        sequence::{preceded, terminated},
    };

    pub fn str<'a>(input: &'a str) -> nom::IResult<&'a str, String, Error> {
        // match rust!
        // use syn::parse_str to parse entire macro invocation
        // might need to slice input before passing it to parse
        let (input, block) = recognize(preceded(
            tag("rust!"),
            terminated(take_till(|c| c == ';'), tag(";")),
        ))(input)?;

        let expression: syn::Macro =
            syn::parse_str(&block[..block.len() - 1]).map_err(|error| {
                nom::Err::Error(Error {
                    input: block.to_string(),
                    kind: ErrorKind::Syntax(format!("{}", error)),
                    backtrace: vec![],
                })
            })?;
        let expression = expression.tokens;

        let block = quote! {
            fn main() -> std::io::Result<()> {
                print!("{}", #expression);
                io::stdout().flush()
            }
        };

        Ok((input, format!("{}", block)))
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
}

mod parse {
    #[derive(Clone)]
    pub enum Segment<'a> {
        Part(&'a str),
        CurrentLocation,
    }

    use nom::{
        branch::alt,
        bytes::complete::{is_not, tag, take_while},
        combinator::{map, value},
        multi::many0,
        sequence::{delimited, preceded},
    };

    pub fn chunk<'a>(input: &'a str) -> nom::IResult<&'a str, String> {
        //let (input, block) =
        //preceded(tag("jago!"), take_while(is_not(value(, newline))))(input)?;

        unimplemented!()
    }

    pub fn acro<'a>(input: &'a str) -> nom::IResult<&'a str, syn::Expr> {
        unimplemented!()
        //let (input, block) = preceded(tag("jago!"), take_while(is_not("\n\n")))(input)?;

        //let parsed = syn::parse_str(block)?;

        //Ok((input, parsed))
    }

    pub fn input<'a>(input: &'a str) -> nom::IResult<&'a str, Vec<Segment<'a>>, Error> {
        /*many0(alt((
            map(take_till(|c| c == ' '), |part| Segment::Part(part)),
            value(
                Segment::CurrentLocation,
                delimited(tag("{"), tag("self"), tag("}")),
            ),
        )))(input)*/

        unimplemented!()
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

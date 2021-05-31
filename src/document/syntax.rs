use std::io::{Read, Write};

pub fn write<'a, R: Read, W: Write>(
    mut reader: R,
    writer: &mut W,
    context: Option<&'a str>,
) -> Result<(), Error> {
    use pulldown_cmark::{html, Event, Parser};

    let mut input = String::new();
    reader.read_to_string(&mut input)?;

    unimplemented!()

    //let parser = Parser::new(&input)
    //.map(|event| match event {
    //Event::Code(code) => parse_code(&code).map(|(_, output)| match output {
    //Code::NameToSentence => {
    //let sentence = context.or(Some("")).map(to_sentence).unwrap();
    //write!(writer, "{}", sentence);
    //true
    //}
    //Code::Text => false,
    //}),
    //_ => {
    //dbg!(&event);
    //Some(false)
    //}
    //})
    //.filter_map(|x| x)
    //.filter(|captured| captured);

    //html::write_html(writer, parser)?;

    //Ok(())
}

//fn parse<'a>(input: &'a str) -> Result<CowStr<'a>, Error> {
//let (_, output) = parse_code(input).map_err(|error: nom::Err<Error>| match error {
//nom::Err::Error(error) | nom::Err::Failure(error) => error,
//nom::Err::Incomplete(needed) => Error::Parse(ParseError {
//input: input.into(),
//kind: ParseErrorKind::Incomplete(needed),
//backtrace: vec![],
//}),
//})?;

//Ok(output.into())
//}

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    combinator::value,
    sequence::delimited,
    IResult,
};

fn parse_code<'a>(input: &'a str) -> IResult<&'a str, Code, ParseError> {
    delimited(
        tag("`"),
        alt((
            value(Code::NameToSentence, tag("%!A a.")),
            value(Code::Text, is_not("`")),
        )),
        tag("`"),
    )(input)
}

#[derive(Clone)]
enum Code {
    NameToSentence,
    Text,
}

fn to_sentence<'a>(input: &'a str) -> String {
    use unicode_segmentation::UnicodeSegmentation;

    let mut output = String::new();

    let mut words = input.unicode_words();

    if let Some(first) = words.next() {
        let mut parts = first.graphemes(true);
        if let Some(first) = parts.next() {
            output.push_str(&first.to_uppercase());
        }
        for part in parts {
            output.push_str(part);
        }
    }

    for word in words {
        output.push_str(word);
    }

    output
}

#[derive(Debug)]
pub enum Error {
    Machine(std::io::Error),
    Parse(ParseError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Machine(error) => write!(f, "{}", error),
            Error::Parse(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Machine(error) => Some(error),
            Error::Parse(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Machine(error)
    }
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Self {
        Self::Parse(error)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParseError {
    input: String,
    kind: ParseErrorKind,
    backtrace: Vec<ParseError>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParseErrorKind {
    Parse(nom::error::ErrorKind),
    Incomplete(nom::Needed),
}

impl nom::error::ParseError<&str> for ParseError {
    fn from_error_kind(input: &str, kind: nom::error::ErrorKind) -> Self {
        ParseError {
            input: input.into(),
            kind: ParseErrorKind::Parse(kind),
            backtrace: vec![],
        }
    }

    fn append(input: &str, kind: nom::error::ErrorKind, mut other: Self) -> Self {
        other.backtrace.push(Self::from_error_kind(input, kind));
        other
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ParseErrorKind::Incomplete(needed) => write!(
                f,
                "incomplete data ({}) input = {}",
                match needed {
                    nom::Needed::Unknown => "unknown".into(),
                    nom::Needed::Size(size) => format!("missing {}", size),
                },
                self.input
            ),
            ParseErrorKind::Parse(kind) => {
                write!(f, "{} - input = {}", kind.description(), self.input)
            }
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

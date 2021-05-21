use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression<'a> {
    Break,
    String(Cow<'a, str>),
    Link(Cow<'a, str>, Cow<'a, str>),
    Combination(Vec<Expression<'a>>),
}

/*
Intro, Jago

Of random company

Pertinent:

[Terms of service.](..)
[Privacy policy.](..)

Other:

[Random.](..kind)
 */
#[test]
fn test_expression() {
    let cases = vec![
        ("Intro, Jago", Expression::String("Intro, Jago".into())),
        ("[a](link)", Expression::Link("a".into(), "link".into())),
        (
            "[Terms of service.](..)",
            Expression::Link("Terms of service.".into(), "terms-of-service".into()),
        ),
        (
            "[Random.](..kind)",
            Expression::Link("Random.".into(), "random-kind".into()),
        ),
        (
            "Intro, Jago

Of random company",
            Expression::Combination(vec![
                Expression::String("Intro, Jago".into()),
                Expression::Break,
                Expression::String("Of random company".into()),
            ]),
        ),
        (
            "Intro, Jago

Of random company

Pertinent:

[Terms of service.](..)
[Privacy policy.](..)

Other:

[Random.](..kind)",
            Expression::Combination(vec![
                Expression::String("Intro, Jago".into()),
                Expression::Break,
                Expression::String("Of random company".into()),
                Expression::Break,
                Expression::String("Pertinent:".into()),
                Expression::Break,
                Expression::Combination(vec![
                    Expression::Link("Terms of service.".into(), "terms-of-service".into()),
                    Expression::Break,
                    Expression::Link("Privacy policy.".into(), "privacy-policy".into()),
                ]),
                Expression::Break,
                Expression::String("Other:".into()),
                Expression::Break,
                Expression::Link("Random.".into(), "random-kind".into()),
            ]),
        ),
    ];

    for (input, want) in cases {
        let (_, got) = expression(input).unwrap();
        assert_eq!(got, want);
    }
}

use nom::{
    branch::alt,
    character::complete::{line_ending, not_line_ending},
    combinator::map,
    multi::fold_many0,
};

fn expression<'a>(input: &'a str) -> nom::IResult<&'a str, Expression<'a>, Error> {
    dbg!(&input);
    let (input, mut expressions) =
        fold_many0(single, vec![], |mut output: Vec<Expression<'_>>, this| {
            match this {
                Expression::Break => match output.last_mut() {
                    None => {}
                    Some(Expression::Break) => {}
                    Some(Expression::Combination(sequence)) => {
                        sequence.push(this);
                    }
                    _ => {
                        //let mut fence = output.len() - 1;
                        //let mut group = vec![this];
                        //let mut broke = false;

                        //loop {
                        //if fence == 0 {
                        //break;
                        //}
                        //match output.get(fence) {
                        //Some(Expression::Break) => {
                        //if broke {
                        //break;
                        //}

                        //broke = true;
                        //}
                        //_ => {
                        //broke = false;
                        //}
                        //};
                        //fence -= 1;
                        //}

                        //dbg!(fence);
                        //dbg!(output.len());
                        //dbg!(&output);
                        //dbg!(&group);

                        //for _ in 0..fence {
                        //if let Some(expression) = output.pop() {
                        //group.push(expression);
                        //}
                        //}

                        //let expression = if group.len() == 1 {
                        //group.pop().unwrap()
                        //} else {
                        //Expression::Combination(group)
                        //};

                        //output.push(expression);

                        //let last = output.pop().unwrap();
                        //output.push(Expression::Combination(vec![last, this]));
                    }
                },
                _ => output.push(this),
            };

            /*
            match output.last_mut() {
                Some(Expression::Combination(combination)) => {
                    combination.push(this);
                }
                Some(Expression::Break) => match this {
                    Expression::Break => {
                        // Much like opinions of others, not all new lines are worth consuming.
                        // They do however help inform the structure at hand.
                    }
                    _ => output.push(Expression::Combination(vec![this])),
                },
                _ => output.push(this),
            };
            */

            /*
            match this {
                Expression::Break => match output.last() {
                    Some(Expression::Break) => match output.last_mut() {
                        Some(Expression::Break) => {
                        }
                        Some(Expression::Combination(inner)) => {
                            inner.push(this);
                        }
                        _ => output.push(this),
                    },
                    _ => output.push(this),
                },
                _ => output.push(this),
            };
            */

            output
        })(input)?;

    let expression = match expressions.len() {
        1 => expressions.pop().unwrap(),
        _ => Expression::Combination(expressions),
    };

    Ok((input, expression))
}

fn single<'a>(input: &'a str) -> nom::IResult<&'a str, Expression<'a>, Error> {
    if input.is_empty() {
        return Err(nom::Err::Error(Error {
            input: input.into(),
            kind: ErrorKind::Incomplete(nom::Needed::new(1)),
            backtrace: vec![],
        }));
    }

    alt((
        map(line_ending, |_| Expression::Break),
        map(link, |(text, destination)| {
            Expression::Link(text, destination)
        }),
        map(not_line_ending, |text: &str| {
            Expression::String(text.into())
        }),
    ))(input)
}

#[test]
fn test_link() {
    let cases = vec![
        ("[a](link)", ("a".into(), "link".into())),
        (
            "[Terms of service.](..)",
            ("Terms of service.".into(), "terms-of-service".into()),
        ),
        (
            "[Random.](..kind)",
            ("Random.".into(), "random-kind".into()),
        ),
    ];

    for (input, want) in cases {
        let (_, got) = link(input).unwrap();
        assert_eq!(got, want);
    }
}

use nom::{
    bytes::complete::{is_not, tag},
    sequence::{delimited, pair},
};

fn link<'a>(input: &'a str) -> nom::IResult<&'a str, (Cow<'a, str>, Cow<'a, str>), Error> {
    let (input, (text, destination)) = pair(
        delimited(tag("["), is_not("]"), tag("]")),
        delimited(tag("("), is_not(")"), tag(")")),
    )(input)?;

    // TODO: make expand optional so it doesn't fail here.
    //
    let (_, destination) = expand(destination, text)?;

    Ok((input, (text.into(), destination.into())))
}

use nom::combinator::rest;

fn expand<'a>(input: &'a str, context: &'a str) -> nom::IResult<&'a str, Cow<'a, str>, Error> {
    use unicode_segmentation::UnicodeSegmentation;

    alt((
        map(pair(tag(".."), rest), |(_, rest): (_, &str)| {
            let context = context.unicode_words().collect::<Vec<&str>>();
            let rest = rest.unicode_words().collect::<Vec<&str>>();
            let words = [context, rest].concat();
            words
                .iter()
                .map(|word| word.to_lowercase())
                .collect::<Vec<_>>()
                .join("-")
                .into()
        }),
        map(rest, |rest: &str| rest.into()),
    ))(input)
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

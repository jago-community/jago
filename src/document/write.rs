use std::io::Write;

use super::Expression;

#[test]
fn test_html() {
    use pretty_assertions::assert_eq;

    let input = "Intro, Jago

Of random company

Pertinent:

[Terms of service.](%)
[Privacy policy.](%)

Other:

[Random.](%-kind)";
    let input = super::parse::unwrapped(input).unwrap();

    let want = "Intro, Jago<br/><br/>\
Of random company<br/><br/>\
Pertinent:<br/><br/>\
<a href=\"terms-of-service\">Terms of service.</a><br/>\
<a href=\"privacy-policy\">Privacy policy.</a><br/><br/>\
Other:<br/><br/>\
<a href=\"random-kind\">Random.</a>";

    let mut got = vec![];

    html(&mut got, input).unwrap();

    assert_eq!(std::str::from_utf8(&got).unwrap(), want);
}

pub fn html<'a, W: Write>(writer: &mut W, expression: Expression<'a>) -> Result<(), Error> {
    match expression {
        Expression::Break => write!(writer, "<br/>")?,
        Expression::String(text) => {
            writer.write(text.as_bytes())?;
        }
        Expression::Link(text, destination) => {
            write!(writer, "<a href=\"{}\">{}</a>", destination, text)?;
        }
        Expression::Combination(block) => {
            for item in block {
                html(writer, item)?;
            }
        }
    };
    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Machine(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Machine(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Machine(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Machine(error)
    }
}

use std::io::Write;

pub fn html<'a, W: Write>(
    mut writer: W,
    title: Option<&'a str>,
    input: &'a str,
) -> Result<(), Error> {
    write!(
        writer,
        r#"
        <!doctype html>
        <html>
            <head>
                <meta charset="utf8">
                <meta name="viewport" content=\"width=device-width, initial-scale=1">
                <title>"#,
    )?;

    if let Some(title) = title {
        writer.write(title.as_bytes())?;
    }

    write!(
        writer,
        r#"
        </title>
        <style>
            ul, ol, p, h1, h2, h3, h4, h5, h6, pre {{
                max-width: 600px;
                margin: 1rem auto 1rem 1rem;
            }}

            code {{
                padding: 6px;
                color: #729B79;
            }}
        </style>
        </head>
        <body>"#
    )?;

    super::format::content(&mut writer, input)?;

    write!(writer, "</body></html>")?;

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Machine(std::io::Error),
    Format(super::format::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Machine(error) => write!(f, "{}", error),
            Error::Format(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Machine(error) => Some(error),
            Error::Format(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::Machine(error)
    }
}

impl From<super::format::Error> for Error {
    fn from(error: super::format::Error) -> Error {
        Error::Format(error)
    }
}

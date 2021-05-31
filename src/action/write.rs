#[derive(Debug, PartialEq)]
pub enum Action {
    Write,
}

pub fn parse<I: Iterator<Item = String>>(input: &mut I) -> Result<Option<Action>, Error> {
    let mut action = None;
    while let Some(item) = input.next() {
        action = match &item[..] {
            "write" => Some(Action::Write),
            _ => None,
        };
    }
    Ok(action)
}

pub fn handle(_action: Option<Action>) -> Result<(), Error> {
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

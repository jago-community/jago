mod action;
mod address;
mod cache;
mod context;
mod environment;
mod image;
mod source;

pub async fn handle<I: Iterator<Item = String>>(input: &mut I) -> Result<(), Error> {
    match environment::populate() {
        Err(error) => {
            eprintln!("error populating environment: {}\n\n", error);
        }
        _ => {}
    };

    let context = context::parse(input)?;

    context::handle(context).await.map_err(Error::from)
}

#[derive(Debug)]
pub enum Error {
    Context(context::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Context(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Context(error) => Some(error),
        }
    }
}

impl From<context::Error> for Error {
    fn from(error: context::Error) -> Self {
        Self::Context(error)
    }
}

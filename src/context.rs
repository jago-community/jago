use crate::action::serve;

pub async fn handle(context: Context) -> Result<(), Error> {
    println!("context {:?}", context);
    match context.action {
        Some(action) => match action {
            Action::Serve => serve::handle().await?,
        },
        None => {
            println!("print help");
        }
    };
    Ok(())
}

#[derive(Debug, PartialEq)]
pub struct Context {
    action: Option<Action>,
}

#[derive(Debug, PartialEq)]
pub enum Action {
    Serve,
}

#[test]
fn test_parse() {
    let cases = vec![
        ("program", None),
        ("program serve       ", Some(Action::Serve)),
    ];

    for (input, want) in cases {
        let got = parse(&mut input.split(" ").map(String::from)).unwrap();
        assert_eq!(got.action, want);
    }
}

pub fn parse<I: Iterator<Item = String>>(input: &mut I) -> Result<Context, Error> {
    let mut action = None;

    let _reference = input.next();

    while let Some(item) = input.next() {
        match &item[..] {
            "serve" => {
                action = Some(Action::Serve);
            }
            _ => {}
        };
    }

    Ok(Context { action })
}

#[derive(Debug)]
pub enum Error {
    Machine(std::io::Error),
    Serve(serve::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Machine(error) => write!(f, "{}", error),
            Error::Serve(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Machine(error) => Some(error),
            Error::Serve(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Machine(error)
    }
}

impl From<serve::Error> for Error {
    fn from(error: serve::Error) -> Self {
        Self::Serve(error)
    }
}

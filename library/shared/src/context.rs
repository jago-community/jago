use std::path::PathBuf;

use crate::{
    action::{output, server, write},
    address::{self, Address},
    cache,
};

use bytes::Bytes;

#[test]
fn test_handle() {
    use tokio_test::block_on;

    let _ = std::fs::remove_file("/tmp/test_handle");

    let input = "program write /tmp/test_handle some stuff";
    let want = "some stuff";

    let mut arguments = input.split(" ").map(String::from);
    let context = parse(&mut arguments).unwrap();

    block_on(async { handle(context).await }).unwrap();

    let got = std::fs::read_to_string("/tmp/test_handle").unwrap();

    assert_eq!(got, want);

    std::fs::remove_file("/tmp/test_handle").unwrap();
}

pub async fn handle(context: Context) -> Result<(), Error> {
    match context.action {
        Some(ref action) => match action {
            Action::Serve => server::handle().await?,
            Action::Store => {}
            Action::Cache(address) => cache::ensure(address).map_err(Error::from)?,
            Action::Write(target, input) => write::handle(target, input)?,
            Action::Output(target, info) => {
                write::handle(&PathBuf::from("/tmp/jago"), info)?;
                output::handle(target)?
            }
        },
        None => {
            println!("print help");
        }
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
pub struct Context {
    action: Option<Action>,
}

#[derive(Debug, PartialEq)]
pub enum Action {
    Serve,
    Store,
    Cache(Address),
    Write(PathBuf, Bytes),
    Output(PathBuf, Bytes),
}

#[test]
fn test_parse() {
    let cases = vec![
        ("program", None),
        ("program serve       ", Some(Action::Serve)),
    ];

    for (input, want) in cases {
        let mut arguments = input.split(" ").map(String::from);
        let got = parse(&mut arguments).unwrap();
        assert_eq!(got.action, want);
    }
}

pub fn parse<I: Iterator<Item = String>>(input: &mut I) -> Result<Context, Error> {
    let mut action = None;

    while let Some(item) = input.next() {
        match &item[..] {
            "serve" => {
                action = Some(Action::Serve);
            }
            "cache" => {
                let maybe_address: String = input.collect();
                let mut address = address::parse(&maybe_address)?;
                address = address.clone();
                action = Some(Action::Cache(address));
            }
            "store" => {
                action = Some(Action::Store);
            }
            "write" => {
                let (target, body) = write::parse(input)?;
                action = Some(Action::Write(target, body));
            }
            "output" => {
                let (target, body) = output::parse(input)?;
                action = Some(Action::Output(target, body));
            }
            _ => {}
        };
    }

    Ok(Context { action })
}

#[derive(Debug)]
pub enum Error {
    Machine(std::io::Error),
    Serve(server::Error),
    Cache(cache::Error),
    Address(address::Error),
    Write(write::Error),
    Output(output::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Machine(error) => write!(f, "{}", error),
            Error::Serve(error) => write!(f, "{}", error),
            Error::Cache(error) => write!(f, "{}", error),
            Error::Address(error) => write!(f, "{}", error),
            Error::Write(error) => write!(f, "{}", error),
            Error::Output(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Machine(error) => Some(error),
            Error::Serve(error) => Some(error),
            Error::Cache(error) => Some(error),
            Error::Address(error) => Some(error),
            Error::Write(error) => Some(error),
            Error::Output(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Machine(error)
    }
}

impl From<server::Error> for Error {
    fn from(error: server::Error) -> Self {
        Self::Serve(error)
    }
}

impl From<cache::Error> for Error {
    fn from(error: cache::Error) -> Self {
        Self::Cache(error)
    }
}

impl From<address::Error> for Error {
    fn from(error: address::Error) -> Self {
        Self::Address(error)
    }
}

impl From<write::Error> for Error {
    fn from(error: write::Error) -> Self {
        Self::Write(error)
    }
}

impl From<output::Error> for Error {
    fn from(error: output::Error) -> Self {
        Self::Output(error)
    }
}

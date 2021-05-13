mod serve;

use crate::request::Request;

pub async fn handle<'a>(request: Request<'a>) -> Result<(), Error> {
    match request {
        Request::Check(Some(inner)) => {
            match inner.as_ref() {
                &Request::Rest(ref maybe_source) => match check(maybe_source) {
                    Err(error) => {
                        eprintln!("check {} failed: {}", maybe_source, error);
                    }
                    _ => {
                        println!("all good");
                    }
                },
                _ => {
                    eprintln!("unexpected pattern following check: {:?}", inner);
                }
            };
        }
        Request::Check(None) => {
            println!("check everything");
        }
        Request::Serve(_) => {
            serve::handle().await?;
        }
        Request::Rest(ref maybe_path) => {
            println!("handle {}", maybe_path);
        }
    };

    Ok(())
}

fn check(source: &str) -> Result<(), Error> {
    let home = dirs::home_dir().unwrap();

    let identity = std::env::var("IDENTITY")
        .or_else(
            |_: std::env::VarError| -> Result<String, Box<dyn std::error::Error>> {
                Ok(String::from(".ssh/id_rsa"))
            },
        )
        .map(|identity| home.join(identity))
        .unwrap();

    let cache = home.join("cache");

    if !cache.exists() {
        match std::fs::create_dir_all(&cache) {
            Err(error) => {
                eprintln!("unexpected error while opening repository: {}", error);
                std::process::exit(1);
            }
            _ => {}
        };
    }

    let path = cache.join(source);

    if let Err(error) = git2::Repository::open(&path) {
        match error.code() {
            git2::ErrorCode::NotFound => {
                use std::io::{stdin, stdout, Write};
                let mut key = String::new();

                print!("identity passphrase: ");

                let _ = stdout().flush();

                if let Err(error) = stdin().read_line(&mut key) {
                    eprintln!("error reading password: {}", error);
                    std::process::exit(1);
                }

                if let Some('\n') = key.chars().next_back() {
                    key.pop();
                }

                if let Some('\r') = key.chars().next_back() {
                    key.pop();
                }

                let mut callbacks = git2::RemoteCallbacks::new();

                callbacks.credentials(|_url, username_from_url, _allowed_types| {
                    git2::Cred::ssh_key(username_from_url.unwrap(), None, &identity, Some(&key))
                });

                let mut fo = git2::FetchOptions::new();
                fo.remote_callbacks(callbacks);

                let mut builder = git2::build::RepoBuilder::new();
                builder.fetch_options(fo);

                match builder.clone(&source, &cache.join(&path)) {
                    Err(error) => {
                        eprintln!("unexpected error while cloning repository: {}", error);
                        std::process::exit(1);
                    }
                    _ => {}
                };
            }
            _ => {
                eprintln!("unexpected error while opening repository: {}", error);
                std::process::exit(1);
            }
        };
    }

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Repository(git2::Error),
    Serve(serve::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Repository(error) => write!(f, "{}", error),
            Error::Serve(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Repository(error) => Some(error),
            Error::Serve(error) => Some(error),
        }
    }
}

impl From<git2::Error> for Error {
    fn from(error: git2::Error) -> Self {
        Self::Repository(error)
    }
}

impl From<serve::Error> for Error {
    fn from(error: serve::Error) -> Self {
        Self::Serve(error)
    }
}

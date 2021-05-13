mod serve;

use bytes::Bytes;

use crate::input::Input;

pub async fn handle<'a>(input: Input<'a>) -> Result<Bytes, Error> {
    let mut output = Bytes::new();

    match input {
        Input::Check(Some(inner)) => {
            match inner.as_ref() {
                &Input::Rest(ref maybe_reference) => match check(maybe_reference) {
                    Err(error) => {
                        eprintln!("check {} failed: {}", maybe_reference, error);
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
        Input::Check(None) => {
            output = check("git@github.com:jago-community/jago.git")?;
        }
        Input::Serve(_) => {
            serve::handle().await?;
        }
        Input::Rest(ref maybe_path) => {
            println!("handle {}", maybe_path);
        }
    };

    Ok(output)
}

#[test]
fn test_handle() {
    {
        // avoid pass phrase checking for keys
        std::fs::create_dir_all(dirs::home_dir().unwrap().join("cache/jago")).unwrap();
        std::fs::copy(
            dirs::home_dir().unwrap().join("local/jago/jago"),
            dirs::home_dir().unwrap().join("cache/jago/jago"),
        )
        .unwrap();
    }

    let got = tokio_test::block_on(handle(Input::Check(None))).unwrap();
    let want = include_str!("../jago");

    assert_eq!(got, want);
}

fn check(maybe_address: &str) -> Result<bytes::Bytes, Error> {
    let address = crate::address::parse(maybe_address)?;
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

    let path = cache.join(address.source());

    if let Err(error) = git2::Repository::open(&path) {
        match error.code() {
            git2::ErrorCode::NotFound => {
                println!("identity key: ");

                let key = rpassword::read_password()?;

                let mut callbacks = git2::RemoteCallbacks::new();

                callbacks.credentials(|_url, username_from_url, _allowed_types| {
                    git2::Cred::ssh_key(username_from_url.unwrap(), None, &identity, Some(&key))
                });

                let mut fo = git2::FetchOptions::new();
                fo.remote_callbacks(callbacks);

                let mut builder = git2::build::RepoBuilder::new();
                builder.fetch_options(fo);

                match builder.clone(&address.source(), &cache.join(&path)) {
                    Err(error) => {
                        println!("{:?} -> {:?}", &address.source(), &cache.join(&path));
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

    println!("got here, maybe check path: {:?}", address.path());

    // if empty path, check for file with same name as repository

    Ok(bytes::Bytes::new())
}

#[derive(Debug)]
pub enum Error {
    Machine(std::io::Error),
    Repository(git2::Error),
    Serve(serve::Error),
    Address(crate::address::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Machine(error) => write!(f, "{}", error),
            Error::Repository(error) => write!(f, "{}", error),
            Error::Serve(error) => write!(f, "{}", error),
            Error::Address(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Machine(error) => Some(error),
            Error::Repository(error) => Some(error),
            Error::Serve(error) => Some(error),
            Error::Address(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Machine(error)
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

impl From<crate::address::Error> for Error {
    fn from(error: crate::address::Error) -> Self {
        Self::Address(error)
    }
}

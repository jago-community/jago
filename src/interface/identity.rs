use std::borrow::Cow;

use openssl::{pkey::PKey, rsa::Rsa};

pub struct Set<'a> {
    pub public: Cow<'a, str>,
    pub private: Cow<'a, str>,
}

pub fn get<'a>() -> Result<Set<'a>, Error> {
    // TODO: read from $IDENTITY.pub file then try this if files not there

    let rsa = Rsa::generate(2048).map_err(|error| Error::Identity(error))?;
    let set = PKey::from_rsa(rsa).map_err(|error| Error::Identity(error))?;

    let public = set.public_key_to_pem()?;
    let private = set.private_key_to_pem_pkcs8()?;

    let public = String::from_utf8(public)?;
    let private = String::from_utf8(private)?;

    Ok(Set {
        public: public.into(),
        private: private.into(),
    })
}

#[derive(Debug)]
pub enum Error {
    Machine(std::io::Error),
    Identity(openssl::error::ErrorStack),
    Cereal(std::string::FromUtf8Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Machine(error) => write!(f, "{}", error),
            Error::Identity(error) => write!(f, "{}", error),
            Error::Cereal(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Machine(error) => Some(error),
            Error::Identity(error) => Some(error),
            Error::Cereal(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::Machine(error)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(error: std::string::FromUtf8Error) -> Error {
        Error::Cereal(error)
    }
}

impl From<openssl::error::ErrorStack> for Error {
    fn from(error: openssl::error::ErrorStack) -> Error {
        Error::Identity(error)
    }
}

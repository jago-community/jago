author::error!(
    Incomplete,
    std::env::VarError,
    std::io::Error,
    std::string::FromUtf8Error,
);

use std::path::{Path, PathBuf};

use secrets::SecretVec;

#[derive(Clone)]
pub struct Identity {
    path: Option<PathBuf>,
    password: Option<Password>,
}

impl Identity {
    pub fn from_variable(name: &str) -> Result<Self, Error> {
        let path = std::env::var(name).map(PathBuf::from)?;

        let mut unprotected_password =
            rpassword::prompt_password_stdout(&format!("Password for {}", path.display()))?
                .into_bytes();

        Ok(Self {
            path: Some(path),
            password: if unprotected_password.len() == 0 {
                None
            } else {
                Some(Password::new(unprotected_password.as_mut_slice()))
            },
        })
    }

    pub fn none() -> Self {
        Self {
            path: None,
            password: None,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn unprotected_password(&self) -> Result<Option<String>, Error> {
        if let Some(ref password) = self.password {
            let unprotected = password.unprotected()?;
            Ok(Some(unprotected))
        } else {
            Ok(None)
        }
    }
}

#[derive(Clone)]
pub struct Password(SecretVec<u8>);

impl Password {
    pub fn new(secret: &mut [u8]) -> Self {
        Self(SecretVec::from(secret))
    }

    pub fn unprotected(&self) -> Result<String, Error> {
        let output = String::from_utf8(Vec::from(&*self.0.borrow()))?;

        Ok(output)
    }
}

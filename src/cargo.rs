use once_cell::sync::OnceCell;

#[derive(Default)]
pub struct Cargo {
    context: OnceCell<Arc<Mutex<Context>>>,
}

use serde::{ser::Error as _, Serialize, Serializer};

impl Serialize for Cargo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let context = self.context().map_err(|error| S::Error::custom(error))?;

        let binaries = context.read_binaries().map_err(S::Error::custom)?;

        binaries.serialize(serializer)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
    #[error("Lock")]
    Lock,
    #[error("Lock")]
    Variable(#[from] std::env::VarError),
}

impl From<&Error> for Error {
    fn from(error: &Error) -> Self {
        match error {
            Error::Lock => Self::Lock,
            Error::Io(error) => Self::Io(std::io::Error::new(error.kind(), "")),
            Error::Variable(error) => Self::Variable(error.clone()),
        }
    }
}

use std::{
    path::PathBuf,
    sync::{Arc, Mutex, PoisonError},
};

impl Error {
    fn lock<Guard>(_: PoisonError<Guard>) -> Self {
        Error::Lock
    }
}

use crate::{Directives, Event, Handle, KeyCode, KeyEvent};

impl Handle for Cargo {
    fn handle(&self, event: &Event) -> Directives {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('?'),
                ..
            }) => match self.context() {
                Ok(mut context) => {
                    context.apply(Op::Read);

                    Directives::empty()
                }
                Err(error) => {
                    log::error!("{}", error);
                    eprintln!("{}", error);

                    Directives::STOP
                }
            },
            _ => Directives::empty(),
        }
    }
}

#[derive(Default, Clone)]
pub struct Context {
    home: OnceCell<Arc<Mutex<Result<PathBuf, Error>>>>,
    binaries: OnceCell<Arc<Mutex<Result<Vec<PathBuf>, Error>>>>,
}

impl Cargo {
    fn context(&self) -> Result<Context, Error> {
        let context = self
            .context
            .get_or_init(|| Default::default())
            .lock()
            .map_err(Error::lock);

        context.as_deref().map(Clone::clone).map_err(Error::from)
    }
}

use crdts::CmRDT;

pub enum Op {
    Read,
}

impl CmRDT for Context {
    type Op = Op;
    type Validation = Error;

    fn validate_op(&self, _: &Self::Op) -> Result<(), Self::Validation> {
        Ok(())
    }

    fn apply(&mut self, op: Self::Op) {
        match op {
            Op::Read => self.read(),
        };
    }
}

impl Context {
    fn read(&self) {
        let _ = self.read_binaries();
    }
}

impl Context {
    fn read_binaries(&self) -> Result<Vec<PathBuf>, Error> {
        let binaries = self
            .binaries
            .get_or_init(|| Arc::new(Mutex::new(self.read_bin_directory())));

        let binaries = binaries.lock().map_err(Error::lock)?;

        binaries
            .as_deref()
            .clone()
            .map(ToOwned::to_owned)
            .map_err(Error::from)
    }

    fn read_bin_directory(&self) -> Result<Vec<PathBuf>, Error> {
        let home = self.read_home_directory()?;

        read_directory(&home)
    }

    fn read_home_directory(&self) -> Result<PathBuf, Error> {
        let home = self
            .home
            .get_or_init(|| {
                Arc::new(Mutex::new(
                    std::env::var("$CARGO_HOME")
                        .map(PathBuf::from)
                        .map(|path| path.join("bin"))
                        .map_err(Error::from),
                ))
            })
            .lock()
            .map_err(Error::lock)?;

        home.as_deref().map(ToOwned::to_owned).map_err(Error::from)
    }
}

use std::path::Path;

fn read_directory(path: &Path) -> Result<Vec<PathBuf>, Error> {
    let directory = std::fs::read_dir(path)?;

    Ok(directory
        .into_iter()
        .flat_map(|result| result.ok())
        .map(|entry| entry.path())
        .collect())
}

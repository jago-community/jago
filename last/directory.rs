use once_cell::sync::OnceCell;

#[derive(Default)]
pub struct Directory {
    context: OnceCell<Arc<Mutex<Context>>>,
}

impl Directory {
    pub fn current() -> Result<Self, Error> {
        use std::env::current_dir;

        current_dir().map_err(Error::from).map(Directory::from)
    }
}

impl From<PathBuf> for Directory {
    fn from(path: PathBuf) -> Self {
        Directory {
            context: OnceCell::from(Arc::new(Mutex::new(Context {
                path,
                ..Default::default()
            }))),
        }
    }
}

use ::{
    serde::{
        ser::{Error as _, SerializeSeq},
        Serialize, Serializer,
    },
    std::path::Path,
};

impl Serialize for Directory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let context = self.context().map_err(|error| S::Error::custom(error))?;

        let entries = context.read_entries().map_err(S::Error::custom)?;

        let mut seq = serializer.serialize_seq(Some(1 + entries.len()))?;

        use std::ffi::OsStr;

        #[inline]
        fn clean(path: &Path) -> &OsStr {
            path.file_name().unwrap_or(&OsStr::new("oops"))
        }

        seq.serialize_element(clean(&context.path))?;

        for e in entries {
            seq.serialize_element(clean(&e))?;
        }

        seq.end()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
    #[error("Lock {0}")]
    Lock(anyhow::Error),
    #[error("Variable {0}")]
    Variable(#[from] std::env::VarError),
}

impl From<&Error> for Error {
    fn from(error: &Error) -> Self {
        match error {
            Error::Lock(g) => Self::Lock(anyhow::anyhow!("{}", g)),
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
    fn lock<Guard>(error: PoisonError<Guard>) -> Self {
        Error::Lock(anyhow::anyhow!("{}", error))
    }
}

use crate::{Directives, Event, Handle, KeyCode, KeyEvent};

impl Handle for Directory {
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

                    Directives::STOP
                }
            },
            _ => self.handle_base(event),
        }
    }
}

#[derive(Default, Clone)]
pub struct Context {
    path: PathBuf,
    entries: OnceCell<Arc<Mutex<Result<Vec<PathBuf>, Error>>>>,
}

impl Directory {
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
        let _ = self.read_entries();
    }
}

impl Context {
    fn read_entries(&self) -> Result<Vec<PathBuf>, Error> {
        let entries = self
            .entries
            .get_or_init(|| Arc::new(Mutex::new(read_directory(&self.path))));

        let entries = entries.lock().map_err(Error::lock)?;

        entries
            .as_deref()
            .clone()
            .map(ToOwned::to_owned)
            .map_err(Error::from)
    }
}

fn read_directory(path: &Path) -> Result<Vec<PathBuf>, Error> {
    let directory = std::fs::read_dir(path)?;

    Ok(directory
        .into_iter()
        .flat_map(|result| result.ok())
        .map(|entry| entry.path())
        .collect())
}

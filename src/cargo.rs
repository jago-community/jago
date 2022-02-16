use std::marker::PhantomData;

#[derive(Default)]
pub struct Cargo(PhantomData<()>);

use serde::{ser::Error as _, Serialize, Serializer};

impl Serialize for Cargo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(context) = self.context() {
            let programs = context.read_programs().map_err(S::Error::custom)?;

            programs.serialize(serializer)?;
        }

        Err(S::Error::custom("no context"))
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
    #[error("NoHome")]
    NoHome,
    #[error("NoPrograms")]
    NoPrograms,
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
            }) => {
                if let Some(mut context) = self.context() {
                    context.apply(Op::Read);

                    Directives::empty()
                } else {
                    log::error!("no context");

                    Directives::empty()
                }
            }
            _ => Directives::empty(),
        }
    }
}

#[derive(Clone)]
pub struct Context;

use once_cell::sync::OnceCell;

impl Cargo {
    fn context(&self) -> Option<Context> {
        static CONTEXT: OnceCell<Arc<Mutex<Context>>> = OnceCell::new();

        let context = CONTEXT
            .get_or_init(|| Arc::new(Mutex::new(Context)))
            .lock()
            .map_err(Error::lock);

        match context {
            Ok(context) => Some(context.deref().clone()),
            Err(_) => None,
        }
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
        let _ = self.read_programs();
    }
}

use std::ops::Deref;

impl Context {
    fn read_programs(&self) -> Result<Vec<PathBuf>, Error> {
        static PROGRAMS: OnceCell<Arc<Mutex<Option<Vec<PathBuf>>>>> = OnceCell::new();

        let programs = PROGRAMS.get_or_init(|| Arc::new(Mutex::new(read_bin_directory().ok())));

        let programs = programs.lock().map_err(Error::lock)?;

        programs.deref().clone().map_or(Err(Error::NoPrograms), Ok)
    }
}

use std::path::Path;

fn read_bin_directory() -> Result<Vec<PathBuf>, Error> {
    static HOME: OnceCell<Arc<Mutex<Option<PathBuf>>>> = OnceCell::new();

    let home = HOME
        .get_or_init(|| {
            Arc::new(Mutex::new(
                std::env::var("$CARGO_HOME")
                    .map(PathBuf::from)
                    .map(|path| path.join("bin"))
                    .ok(),
            ))
        })
        .lock()
        .map_err(Error::lock)?;

    match home.deref() {
        Some(path) => read_directory(&path),
        _ => Err(Error::NoHome),
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

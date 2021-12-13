use crate::document;

use std::sync::{Arc, Mutex};

use document::Document;

#[derive(Default)]
pub struct Context {
    document: Arc<Mutex<Document>>,
    inner: Arc<Mutex<Vec<u8>>>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Poisoned")]
    Poisoned,
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
    #[error("Document {0}")]
    Document(#[from] document::Error),
    #[error("SetLogger")]
    SetLogger,
    #[error("AfterBefore")]
    AfterBefore,
}

use std::io::Write;

impl Context {
    pub fn write(&self, input: impl AsRef<[u8]>) -> Result<(), Error> {
        let mut inner = self.inner.lock().map_err(|_| Error::Poisoned)?;

        inner.write_all(input.as_ref()).map_err(Error::from)
    }

    pub fn target(&self) -> Vec<u8> {
        if let Ok(inner) = self.inner.lock() {
            Vec::with_capacity(inner.len())
        } else {
            Vec::new()
        }
    }

    pub fn read(&self, mut target: impl Write) -> Result<(), Error> {
        let inner = self.inner.lock().map_err(|_| Error::Poisoned)?;

        target.write_all(&inner)?;

        Ok(())
    }
}

use log::{Log, Metadata, Record};

use document::Expression;

impl Log for Context {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if let Ok(mut document) = self.document.lock() {
            if let Err(error) = document.record(Expression::Log(record)) {
                eprintln!("unable to record log: {}", error);
            }
        }
    }

    fn flush(&self) {}
}

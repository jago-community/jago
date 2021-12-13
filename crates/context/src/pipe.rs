use std::{
    io::{Stderr, Write},
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct Pipe<Inner> {
    inner: Arc<Mutex<Vec<u8>>>,
    steps: Arc<Mutex<Vec<usize>>>,
    target: Inner,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Poisoned")]
    Poisoned,
}

impl From<Stderr> for Pipe<Stderr> {
    fn from(stderr: Stderr) -> Self {
        Pipe {
            inner: Arc::new(Mutex::new(Vec::new())),
            steps: Arc::new(Mutex::new(Vec::new())),
            target: stderr,
        }
    }
}

use std::sync::MutexGuard;

impl Pipe<Stderr> {
    pub fn inner<'a>(&'a self) -> Result<MutexGuard<'a, impl Write>, Error> {
        self.inner.lock().map_err(|_| Error::Poisoned)
    }
}

impl<W: Write> Write for Pipe<W> {
    fn write(&mut self, input: &[u8]) -> std::io::Result<usize> {
        let written = self.target.write(input)?;

        if let Ok(mut inner) = self.inner.lock() {
            inner.extend_from_slice(&input[0..written]);

            if let Ok(mut steps) = self.steps.lock() {
                steps.push(inner.len());
            }
        }

        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.target.flush()
    }
}

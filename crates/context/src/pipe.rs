use std::{
    io::{Stderr, Write},
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct Pipe<Inner> {
    inner: Vec<u8>,
    steps: Vec<usize>,
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
            inner: Vec::new(),
            steps: Vec::new(),
            target: stderr,
        }
    }
}

impl<W: Write> Write for Pipe<W> {
    fn write(&mut self, input: &[u8]) -> std::io::Result<usize> {
        let written = self.target.write(input)?;

        self.steps.push(written);
        self.inner.extend_from_slice(&input[0..written]);

        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.target.flush()
    }
}

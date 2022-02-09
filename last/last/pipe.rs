pub struct Pipe<Inner> {
    inner: Inner,
}

use std::io::{self, Write};

impl<W: Write> Write for Pipe<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

use crossterm::{Command, QueueableCommand};

impl<W: Write> Pipe<W> {
    fn take(&mut self, directive: impl Command) {
        self.queue(directive) // .unwrap();
    }
}

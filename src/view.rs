pub struct Buffer<Data> {
    inner: Data,
}

use std::fmt;

impl<W: fmt::Write> fmt::Write for Buffer<W> {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.inner.write_str(s)
    }

    fn write_char(&mut self, c: char) -> Result<(), fmt::Error> {
        self.inner.write_char(c)
    }

    fn write_fmt(&mut self, a: fmt::Arguments<'_>) -> Result<(), fmt::Error> {
        self.inner.write_fmt(a)
    }
}

use crossterm::Command;

impl<D: Command> Command for Buffer<D> {
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        self.inner.write_ansi(out)
    }
}

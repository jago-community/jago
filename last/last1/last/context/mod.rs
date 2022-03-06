mod buffer;
mod context;
mod directive;
mod traits;
mod watch;

pub use context::get;

/*
pub struct Context<Buffer> {
    buffer: Buffer,
}

impl<B> Context<B> {
    fn watch(&mut self) -> usize {
        0
    }
}

use std::io::{self, BufWriter, Write};

impl<W: Write> Write for Context<BufWriter<W>> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buffer.flush()
    }
}

use std::fmt::{self, Write as Fmt};

impl<F: Fmt> Fmt for Context<F> {
    fn write_str(&mut self, buf: &str) -> fmt::Result {
        self.buffer.write_str(buf)
    }
}
*/

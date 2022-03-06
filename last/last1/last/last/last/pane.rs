pub struct Pane<Inner> {
    inner: Inner,
}

use std::fmt;

use crossterm::{Command, style::Print};

impl Command {
    fn write_ansi(&self, out: &mut fmt::Write) -> fmt::Result; {
        Print("Hello, stranger.").write_ansi(out)
    }
}

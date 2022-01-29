pub struct Plane<Inner> {
    inner: Inner,
    dimensions: (usize, usize),
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
}

impl<Inner> Plane<Inner> {
    pub fn with_dimensions(inner: Inner, (x, y): (usize, usize)) -> Self {
        Plane {
            inner,
            dimensions: (x, y),
        }
    }
}

use crossterm::Command;

use std::fmt;

impl<Inner: Command> Command for Plane<Inner> {
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        self.inner.write_ansi(out)
    }
}

use crate::traits::Viewer;

impl<Inner: Command> Viewer for Plane<Inner> {}

use crossterm::event::Event;

use crate::traits::{Handler, Outcome};

impl<Inner> Handler for Plane<Inner> {
    fn handle(&mut self, event: &Event) -> Outcome {
        self.handle_common(event)
    }
}

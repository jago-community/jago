use std::path::Path;

pub struct Resource<'a>(&'a Path);

impl<'a> From<&'a Path> for Resource<'a> {
    fn from(path: &'a Path) -> Self {
        Resource(path)
    }
}

use crossterm::{style::Print, Command};

impl Command for Resource<'_> {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        Print(self.0.display()).write_ansi(out)
    }
}

use crate::handle::{Handle, Outcome};

use crossterm::event::{Event, KeyCode, KeyEvent};

impl Handle for Resource<'_> {
    fn handle(&mut self, event: &Event) -> Outcome {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => {
                // ...
                Outcome::Done
            }
            _ => self.handle_common(event),
        }
    }
}

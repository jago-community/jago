use std::path::Path;

pub struct Resource<'a>(&'a Path);

impl<'a> From<&'a Path> for Resource<'a> {
    fn from(path: &'a Path) -> Self {
        Resource(path)
    }
}

use crate::traits::{Lense, Viewer};

impl Viewer for Resource<'_> {
    fn view(&self) -> Lense {
        Lense::Encoded(Box::new(self.0.display()))
    }
}

use crate::traits::{Handler, Outcome};

use crossterm::event::{Event, KeyCode, KeyEvent};

impl Handler for Resource<'_> {
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

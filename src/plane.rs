pub struct Plane<Inner> {
    inner: Inner,
    dimensions: (usize, usize),
    alignment: Alignment,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
}

enum Alignment {
    Start,
    Center,
    End,
}

impl Alignment {
    fn next(&self) -> Alignment {
        match self {
            &Alignment::Start => Alignment::Center,
            &Alignment::Center => Alignment::End,
            &Alignment::End => Alignment::Start,
        }
    }
}

impl<Inner> Plane<Inner> {
    pub fn with_dimensions(inner: Inner, (x, y): (usize, usize)) -> Self {
        Plane {
            inner,
            dimensions: (x, y),
            alignment: Alignment::Start,
        }
    }
}

use crate::traits::{Lense, Viewer};

impl<'a, Inner: Viewer> Viewer for &'a Plane<Inner> {
    fn view(&self) -> Lense {
        Lense::Group(vec![
            Lense::Cursor((0, (self.dimensions.0 / 2, 0))),
            self.inner.view(),
        ])
    }
}

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::traits::{Handler, Outcome};

impl<Inner> Handler for Plane<Inner> {
    fn handle(&mut self, event: &Event) -> Outcome {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => {
                self.alignment = self.alignment.next();
                Outcome::Continue
            }
            _ => self.handle_common(event),
        }
    }
}

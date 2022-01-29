pub struct Sequence<'a> {
    sequence: Vec<Box<dyn Viewer + 'a>>,
    step: usize,
}

impl Sequence<'_> {
    pub fn wrap<'a, V>(item: &'a V) -> Box<dyn Viewer + 'a>
    where
        &'a V: Viewer,
    {
        Box::new(item)
    }
}

impl<'a> From<Vec<Box<dyn Viewer + 'a>>> for Sequence<'a> {
    fn from(sequence: Vec<Box<dyn Viewer + 'a>>) -> Self {
        Self { sequence, step: 0 }
    }
}

use crate::traits::{Lense, Viewer};

impl Viewer for Sequence<'_> {
    fn view(&self) -> Lense<'_> {
        self.sequence
            .get(self.step)
            .map(|item| item.view())
            .unwrap_or_else(|| Lense::Encoded(Box::new("shouldn't see")))
    }
}

use crate::traits::{Handler, Outcome};

use crossterm::event::{Event, KeyCode, KeyEvent};

impl Handler for Sequence<'_> {
    fn handle(&mut self, event: &Event) -> Outcome {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => {
                if self.sequence.len() - 1 > self.step {
                    if let Some(next) = self.step.checked_add(1) {
                        self.step = next;
                        Outcome::Continue
                    } else {
                        Outcome::Exit(Some(1))
                    }
                } else {
                    Outcome::Done
                }
            }
            _ => self.handle_common(event),
        }
    }
}

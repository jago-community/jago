use crate::filter::Filter;

pub struct Sequence<'a> {
    sequence: Vec<Box<dyn Filter + 'a>>,
    step: usize,
}

impl Sequence<'_> {
    pub fn wrap<'a>(item: impl Filter + 'a) -> Box<dyn Filter + 'a> {
        Box::new(item)
    }
}

impl<'a> From<Vec<Box<dyn Filter + 'a>>> for Sequence<'a> {
    fn from(sequence: Vec<Box<dyn Filter + 'a>>) -> Self {
        Self { sequence, step: 0 }
    }
}

use crate::view::{Op, View};

impl View for Sequence<'_> {
    fn view(&self) -> Op<'_> {
        self.sequence
            .get(self.step)
            .map(|item| item.view())
            .unwrap_or(Op::Empty)
    }
}

use crate::handle::{Handle, Outcome};

use crossterm::event::{Event, KeyCode, KeyEvent};

impl Handle for Sequence<'_> {
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

    fn handle_inner(&mut self, event: &Event) -> Outcome {
        self.sequence
            .get_mut(self.step)
            .map(|item| item.handle(event))
            .unwrap_or(Outcome::Continue)
    }
}

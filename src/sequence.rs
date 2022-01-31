use crate::handle::Handle;

pub struct Sequence<'a> {
    sequence: Vec<Box<dyn Display + 'a>>,
    step: usize,
}

impl Sequence<'_> {
    pub fn wrap<'a>(item: impl Display + 'a) -> Box<dyn Display + 'a> {
        Box::new(item)
    }
}

impl<'a> From<Vec<Box<dyn Display + 'a>>> for Sequence<'a> {
    fn from(sequence: Vec<Box<dyn Display + 'a>>) -> Self {
        Self { sequence, step: 0 }
    }
}

use std::fmt::Display;

impl Display for Sequence<'_> {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.sequence
            .get(self.step)
            .map(|item| item.fmt(out))
            .unwrap_or(Err(std::fmt::Error))
    }
}

use crossterm::{style::Print, Command};

impl Command for Sequence<'_> {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        self.sequence
            .get(self.step)
            .map(|item| Print(item).write_ansi(out))
            .unwrap_or(Err(std::fmt::Error))
    }
}

use crossterm::event::{Event, KeyCode, KeyEvent};

use crate::handle::Outcome;

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
}

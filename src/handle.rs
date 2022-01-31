#[derive(PartialEq)]
pub enum Outcome {
    Continue,
    Done,
    Exit(Option<i32>),
}

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

pub trait Handle {
    fn handle(&mut self, event: &Event) -> Outcome {
        self.handle_common(event)
    }

    fn handle_common(&mut self, event: &Event) -> Outcome {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => Outcome::Exit(None),
            _ => Outcome::Continue,
        }
    }

    fn handle_inner(&mut self, _: &Event) -> Outcome {
        Outcome::Continue
    }

    fn handle_event(&mut self, event: &Event) -> Outcome {
        match self.handle_inner(event) {
            Outcome::Continue => self.handle(event),
            outcome @ _ => outcome,
        }
    }
}

impl Handle for &str {}

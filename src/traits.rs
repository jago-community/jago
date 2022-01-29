#[derive(PartialEq)]
pub enum Outcome {
    Continue,
    Done,
    Exit(Option<i32>),
}

use crossterm::event::Event;

pub trait Handler {
    fn handle(&mut self, event: &Event) -> Outcome;

    fn handle_common(&mut self, event: &Event) -> Outcome {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => Outcome::Exit(None),
            _ => Outcome::Continue,
        }
    }
}

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crossterm::Command;

pub trait Viewer: Command {}

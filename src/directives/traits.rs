#[derive(PartialEq)]
pub enum Op {
    Continue,
    Done,
    Exit(Option<i32>, Option<String>),
}

pub use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    Command,
};

pub trait Directive: Command {
    fn handle(&mut self, event: &Event) -> Op {
        self.handle_common(event)
    }

    fn handle_common(&mut self, event: &Event) -> Op {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => Op::Exit(None, None),
            _ => Op::Continue,
        }
    }

    fn handle_inner(&mut self, _: &Event) -> Op {
        Op::Continue
    }

    fn handle_event(&mut self, event: &Event) -> Op {
        match self.handle_inner(event) {
            Op::Continue => self.handle(event),
            op @ _ => op,
        }
    }
}

impl<D: Directive> Directive for &D {}

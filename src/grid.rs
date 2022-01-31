pub struct Grid {
    // ./..
    x: (usize, usize),
    y: (usize, usize),
}

use crossterm::Command;

pub trait View: Command {
    fn view(&self, buffer: &mut Grid);
}

#[derive(PartialEq)]
pub enum Outcome {
    Continue,
    Done,
    Exit(Option<i32>),
}

use crossterm::event::Event;

pub trait Handle {
    fn handle(&mut self, event: &Event) -> Outcome {
        self.handle_common(event)
    }

    fn handle_common(&mut self, event: &Event) -> Outcome {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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

// pub trait Screen: View + Handle {
//   fn filter<Buffer>(&self, buffer: &mut Grid<Buffer>) -> Result<(), ()>;
// }

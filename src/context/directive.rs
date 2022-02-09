#[derive(PartialEq)]
pub enum Handle {
    Continue,
    Done,
    Exit(Handletion<i32>, Handletion<String>),
}

impl Handle {
    pub fn stop(&self) -> bool {
        match self {
            Handle::Done | Handle::Exit(_, _) => true,
            _ => false,
        }
    }
}

pub use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    Command,
};

pub trait Directive {
    fn before(&mut self) -> Result<(), Box<dyn std::error::Error + 'static>> {
        Ok(())
    }

    fn handle(&mut self, event: &Event) -> Handle {
        self.handle_common(event)
    }

    fn handle_common(&mut self, event: &Event) -> Handle {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => Handle::Exit(None, None),
            _ => Handle::Continue,
        }
    }

    fn handle_inner(&mut self, _: &Event) -> Handle {
        Handle::Continue
    }

    fn handle_event(&mut self, event: &Event) -> Handle {
        match self.handle_inner(event) {
            Handle::Continue => self.handle(event),
            handle @ _ => handle,
        }
    }

    fn cloned(self: &Self) -> Self
    where
        Self: Clone,
    {
        self.clone()
    }
}

impl<D: Directive> Directive for &D {}

use bitflags::bitflags;

bitflags! {
    pub struct Directives: u32 {
        const STOP = 0b00000001;
    }
}

impl Directives {
    pub fn stop(&self) -> bool {
        self.contains(Directives::STOP)
    }
}

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

pub trait Handle {
    fn handle(&self, event: &Event) -> Directives {
        self.handle_base(event)
    }

    fn handle_base(&self, event: &Event) -> Directives {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
            }) => Directives::STOP,
            _ => Directives::empty(),
        }
    }
}

impl<S: Sized> Handle for S {}

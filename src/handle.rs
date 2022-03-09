use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

pub trait Handle {
    fn handle(&self, event: &Event) -> Directives {
        self.handle_base(event)
    }

    fn handle_base(&self, event: &Event) -> Directives {
        log::info!("{:?}", event);

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
            }) => Directives::STOP,
            _ => Directives::empty(),
        }
    }
}

pub trait Directive {
    fn stop(&self) -> bool;
}

use bitflags::bitflags;

bitflags! {
    pub struct Directives: u32 {
        const STOP = 0b00000001;
    }
}

impl Directive for Directives {
    fn stop(&self) -> bool {
        self.contains(Directives::STOP)
    }
}

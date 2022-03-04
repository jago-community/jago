pub struct Window<S> {
    inner: S,
    size: (u16, u16),
    offsets: (u16, u16),
}

use crate::{Directives, Event, Handle, KeyCode, KeyEvent};

impl<S> Handle for Window<S> {
    fn handle(&self, event: &Event) -> Directives {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                ..
            }) => Directives::STOP,
            Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                ..
            }) => Directives::STOP,
            _ => Directives::empty(),
        }
    }
}

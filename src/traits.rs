use crossterm::event::Event;

pub trait Handler {
    fn handle(&mut self, event: &Event) -> Op;

    fn handle_common(&mut self, event: &Event) -> Op {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => Outcome::Exit(None),
            _ => Outcome::Continue,
        }
    }
}

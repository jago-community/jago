use crossterm::event::Event;

pub trait Handler {
    fn handle(event: &Event);
}

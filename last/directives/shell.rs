pub struct Shell<D>(D, (u16, u16), u16);

impl<D> Shell<D> {
    pub fn new(d: D) -> Self {
        Self(d, (0, 0), 80)
    }
}

use super::traits::{Command, Directive, Event, Op};

impl<D: Directive> Directive for Shell<D> {
    fn handle(&mut self, event: &Event) -> Op {
        match event {
            Event::Resize(x, y) => {
                self.1 = (*x, *y);
                Op::Continue
            }
            _ => self.handle_common(event),
        }
    }

    fn handle_inner(&mut self, event: &Event) -> Op {
        self.0.handle_event(event)
    }
}

use ::{
    crossterm::{
        cursor::MoveTo,
        style::{Color, Print, SetForegroundColor},
        terminal::{Clear, ClearType},
    },
    std::fmt,
};

impl<D: Directive> Command for Shell<D> {
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        //let x_offset = (self.1 .0 / 2) - self.2 / 2;

        Clear(ClearType::All)
            .write_ansi(out)
            .and(MoveTo(0, 0).write_ansi(out))
            .and(SetForegroundColor(Color::Green).write_ansi(out))
            .and(Print("> ").write_ansi(out))
            .and(SetForegroundColor(Color::Reset).write_ansi(out))
            .and(self.0.write_ansi(out))
    }
}

pub struct Plane<Inner> {
    inner: Inner,
    dimensions: (usize, usize),
    point: (usize, usize),
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
}

impl<Inner> Plane<Inner> {
    pub fn with_dimensions(inner: Inner, (x, y): (usize, usize)) -> Self {
        Plane {
            inner,
            dimensions: (x, y),
            point: (0, 0),
        }
    }

    fn step(&mut self, (x, y): (isize, isize)) {
        if x.is_positive() {
            if let Some(next) = self.point.0.checked_add(x.unsigned_abs()) {
                self.point.0 = next;
            }
        } else {
            if let Some(next) = self.point.0.checked_sub(x.unsigned_abs()) {
                self.point.0 = next;
            }
        }

        if y.is_positive() {
            if let Some(next) = self.point.1.checked_add(y.unsigned_abs()) {
                self.point.1 = next;
            }
        } else {
            if let Some(next) = self.point.1.checked_sub(y.unsigned_abs()) {
                self.point.1 = next;
            }
        }
    }
}

use std::fmt::Display;

//impl<Inner: Display> Display for Plane<Inner> {
//fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//self.inner.fmt(out)
//}
//}

//use num_traits::FromPrimitive;

//use crossterm::{cursor::MoveTo, style::Print, Command};

//impl<Inner: Command> Command for Plane<Inner> {
//fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
//MoveTo(x, y).write_ansi(out).and(self.inner.write_ansi(out))
//}
//}

use crate::view::{self, Op, View};

impl<Inner: Display> View for Plane<Inner> {
    fn view(&self) -> Op<'_> {
        Op::And(
            Box::new(Op::Cursor(self.point.0, self.point.1)),
            Box::new(Op::from(&self.inner)),
        )
    }
}

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::handle::{Handle, Outcome};

impl<Inner: Handle> Handle for Plane<Inner> {
    fn handle(&mut self, event: &Event) -> Outcome {
        self.handle_own(event)
    }

    fn handle_inner(&mut self, event: &Event) -> Outcome {
        self.inner.handle(event)
    }
}

impl<Inner: Handle> Plane<Inner> {
    fn handle_own(&mut self, event: &Event) -> Outcome {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('h'),
                ..
            }) => {
                self.step((-1, 0));

                Outcome::Continue
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                ..
            }) => {
                self.step((0, 1));

                Outcome::Continue
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                ..
            }) => {
                self.step((0, -1));

                Outcome::Continue
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                ..
            }) => {
                self.step((1, 0));

                Outcome::Continue
            }
            _ => self.handle_common(event),
        }
    }
}

use crate::filter::Filter;

impl<Inner: Display + Handle> Filter for Plane<Inner> {}

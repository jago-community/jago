pub trait View {
    fn view(&self) -> Op<'_>;
}

use std::fmt::Display;

impl<T: Display> View for T {
    fn view(&self) -> Op<'_> {
        Op::from(self)
    }
}

pub enum Op<'a> {
    Print(Box<dyn Display + 'a>),
    Cursor(usize, usize),
    And(Box<Op<'a>>, Box<Op<'a>>),
    Empty,
}

impl<'a, I: Display + 'a> From<I> for Op<'a> {
    fn from(i: I) -> Self {
        Self::Print(Box::new(i))
    }
}

use crossterm::{cursor::MoveTo, style::Print, Command};

use num_traits::FromPrimitive;

impl Command for Op<'_> {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            Op::Print(inner) => Print(inner).write_ansi(out),
            Op::Cursor(x, y) => {
                let x = u16::from_usize(*x).ok_or(std::fmt::Error)?;
                let y = u16::from_usize(*y).ok_or(std::fmt::Error)?;
                MoveTo(x, y).write_ansi(out)
            }
            Op::And(a, b) => a.write_ansi(out).and(b.write_ansi(out)),
            Op::Empty => Ok(()),
        }
    }
}

/*
pub struct View<'a, Inner> {
    view: Box<dyn Fn() -> Box<Inner> + 'a>,
}

use crossterm::Command;

impl<'a, Inner: Command> Command for View<'a, Inner> {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        self.view.as_ref()().write_ansi(out)
    }
}

use std::fmt::Display;

use crossterm::style::Print;

impl<'a, Inner> From<Print<&'a Inner>> for View<'a, Print<&'a Inner>>
where
    &'a Inner: Display,
{
    fn from(this: Print<&'a Inner>) -> Self {
        Self {
            view: Box::new(move || Box::new(this)),
        }
    }
}*/

use std::fmt;

pub trait View {
    fn view(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result;
}

//impl<I: fmt::Display> View for I {
//fn view<'a>(&self, out: &mut fmt::Formatter<'a>) -> fmt::Result {
//self.fmt(out)
//}
//}

use crossterm::Command;

impl<I: Command> View for I {
    fn view<'a>(&self, out: &mut fmt::Formatter<'a>) -> fmt::Result {
        self.write_ansi(out)
    }
}

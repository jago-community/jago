/*
pub struct Cell<C> {
    inner: C,
}


impl<C: Command> Command for Cell<C> {
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        self.inner.write_ansi(out)
    }
}

use crossterm::style::Print;

impl<'a> From<&'a str> for Cell<Print<&'a str>> {
    fn from(s: &'a str) -> Self {
        Cell { inner: Print(s) }
    }
}

impl<'a> From<((u16, u16), &'a str)> for Cell<Print<&'a str>> {
    fn from(s: ((u16, u16), &'a str)) -> Self {
        Cell { inner: Print(s.1) }
    }
}

pub struct Group<C> {
    inner: C,
}

impl<C: Command> Command for Group<C>
where
    C: Iterator,
    C::Item: Command,
{
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        self.inner
            .clone()
            .into_iter()
            .map(|i| i.write_ansi(out))
            .fold(Ok(()), |_, next| next)
    }
}
*/

use std::fmt;

use crossterm::Command;

pub struct Sequence<'a, V>(&'a V);

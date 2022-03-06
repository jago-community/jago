#[derive(Default)]
pub struct Buffer<'a> {
    directives: &'a [Directive<'a, String>],
}

impl<'a> From<&'a [Directive<'a, String>]> for Buffer<'a> {
    fn from(directives: &'a [Directive<'a, String>]) -> Self {
        Self { directives }
    }
}

use crossterm::Command;

use std::fmt;

impl<'a> Command for Buffer<'a> {
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        let accum = self.directives.iter().fold(String::new(), |mut a, b| {
            b.call(&mut a);

            a
        });

        out.write_str(&accum)
    }
}

pub struct Directive<'a, B> {
    f: Box<dyn 'a + Fn(&mut B) -> fmt::Result>,
}

impl<'a, C, B> From<C> for Directive<'a, B>
where
    C: Command + 'a,
    B: fmt::Write,
{
    fn from(c: C) -> Self {
        Self {
            f: Box::new(move |mut b| c.write_ansi(&mut b)),
        }
    }
}

impl<'a, B> Directive<'a, B>
where
    B: fmt::Write,
{
    pub fn wrap(c: impl Command + 'a) -> Self {
        Self {
            f: Box::new(move |mut b| c.write_ansi(&mut b)),
        }
    }

    fn call(&self, out: &mut B) -> fmt::Result {
        (*self.f)(out)
    }
}

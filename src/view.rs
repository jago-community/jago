pub struct Buffer<'a, Data> {
    inner: Data,
    directives: Vec<Directive<'a, String>>,
}

use std::fmt;

impl<W: fmt::Write> fmt::Write for Buffer<W> {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.inner.write_str(s)
    }

    fn write_char(&mut self, c: char) -> Result<(), fmt::Error> {
        self.inner.write_char(c)
    }

    fn write_fmt(&mut self, a: fmt::Arguments<'_>) -> Result<(), fmt::Error> {
        self.inner.write_fmt(a)
    }
}

use crossterm::Command;

impl<'a, D> Command for Buffer<D>
where
    D: IntoIterator<Item = Directive<'a, Buffer<String>>>,
{
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        let mut accum = Buffer {
            inner: String::new(),
        };

        for d in self.inner {
            d.call(&mut accum)?;
        }

        Ok(())
    }
}

pub struct Directive<'a, B> {
    f: &'a dyn Fn(&mut B) -> fmt::Result,
}

impl<'a, B> Directive<'a, B>
where
    B: fmt::Write,
{
    fn wrap(c: impl Command + 'a) -> Self {
        Self {
            f: &move |&mut b| c.write_ansi(&mut b),
        }
    }

    fn call(self, out: &mut B) -> fmt::Result {
        (*self.f)(out)
    }
}

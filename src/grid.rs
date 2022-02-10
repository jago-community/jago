pub struct Grid<'a, I> {
    buffer: I,
    ph: std::marker::PhantomData<&'a ()>,
}

impl<'a, I> From<I> for Grid<'a, I> {
    fn from(buffer: I) -> Self {
        Grid {
            buffer,
            ph: Default::default(),
        }
    }
}

use ::{
    crossterm::{cursor::MoveToNextLine, style::Print, Command},
    itertools::{FoldWhile, Itertools},
    std::fmt,
};

impl<'a, I> Command for Grid<'a, I>
where
    I: IntoIterator<Item = char>,
{
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        self.buffer
            .into_iter()
            .map(|ch| match ch {
                '\n' => MoveToNextLine(1).write_ansi(out),
                _ => Print(ch).write_ansi(out),
            })
            .fold_while(Ok(()), |_, next| {
                if next.is_ok() {
                    FoldWhile::Continue(Ok(()))
                } else {
                    FoldWhile::Done(Err(fmt::Error))
                }
            })
            .into_inner()
    }
}

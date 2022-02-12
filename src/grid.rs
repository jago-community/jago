pub struct CharGrid {
    buffer: Vec<char>,
}

impl CharGrid {
    pub fn new(buffer: Vec<char>) -> Self {
       Self {
            buffer,
       }
    }
}

use ::{std::fmt, itertools::{Itertools, FoldWhile}, crossterm::{style::Print, Command, cursor::MoveToNextLine}};

impl Command for CharGrid {
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        self.buffer
            .iter()
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

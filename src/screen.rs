use crate::handle::Handle;

use crossterm::Command;

pub trait Screen: Command + Handle {}

impl<Buffer: Handle + Command> Screen for Buffer {}

use std::borrow::Cow;

pub struct Cells<'a> {
    buffer: Cow<'a, str>,
}

impl crate::handle::Handle for Cells<'_> {}

impl<'a> From<&'a str> for Cells<'a> {
    fn from(buffer: &'a str) -> Self {
        Self {
            buffer: buffer.into(),
        }
    }
}

use unicode_segmentation::UnicodeSegmentation;

impl Cells<'_> {
    fn steps(&self) -> impl Iterator<Item = (&str, (usize, usize))> {
        self.buffer
            .graphemes(true)
            .map(|grapheme| (grapheme, (0, 0)))
    }
}

use crossterm::style::Print;
use itertools::{FoldWhile, Itertools};

impl Command for Cells<'_> {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        self.steps()
            .scan((0, 0), |(dx, dy), (grapheme, (x, y))| {
                Some((grapheme, (x, y)))
            })
            .map(|(grapheme, _)| Print(grapheme).write_ansi(out))
            .fold_while(Ok(()), |_, this| {
                if this.is_ok() {
                    FoldWhile::Continue(this)
                } else {
                    FoldWhile::Done(this)
                }
            })
            .into_inner()
    }
}

pub struct Grid<'a> {
    buffer: Cells<'a>,
}

use std::fmt::Display;

pub trait View<'a> {
    type Item: Display;

    type Items: Iterator<Item = Self::Item>;

    fn split(&'a self) -> Self::Items;

    fn grid(&'a self) -> Grid<'a, Self::Items> {
        Grid {
            parts: &self.split(),
        }
    }
}

use unicode_segmentation::{Graphemes, UnicodeSegmentation};

impl<'a> View<'a> for &'a str {
    type Item = Self;

    type Items = Graphemes<'a>;

    fn split(&self) -> Self::Items {
        self.graphemes(true)
    }
}

pub struct Grid<'a, Parts> {
    parts: &'a Parts,
}

use crate::handle::Handle;

pub trait Screen<'a>: View<'a> + Handle {}

impl<'a> Screen<'a> for &'a str {}

/*
impl<'a> View<'a> for &'a str {
    type Part = Self;
    type Segments = Graphemes<'a>;

    fn cells(&self) -> Self::Segments {
        self.graphemes(true)
    }
}
*/

//pub trait View<'a> {
//type Items: Iterator<Item = &'a str>;

//type Segments: UnicodeSegmentation;

//fn cells(&'a self) -> Cells<'a, Self::Items>;

//fn segments(&'a self) -> ;

/*
fn grid(
    &self,
) -> std::iter::Map<Self::Filter, &'a dyn FnMut(&'a str) -> (&'a str, (usize, usize))>;
*/
//}

//pub struct Cells<'a, Buffer> {
//buffer: &'a Buffer,
//}

//impl<'a, Buffer> Handle for Cells<'a, Buffer> {}

//use unicode_segmentation::UnicodeSegmentation;

//impl<'a, B> From<&'a B> for Cells<'a, B>
//where
//B: UnicodeSegmentation,
//{
//fn from(buffer: &'a B) -> Self {
//Self { buffer }
//}
//}

//impl<'a, Buffer> View<'a> for Cells<'a, Buffer>
//where
//Buffer: UnicodeSegmentation,
//{
//type Items = Graphemes<'a>;

//fn cells(&'a self) -> Self::Items {
//self.buffer.graphemes(true)
//}
//}

use crossterm::{style::Print, Command};

use itertools::{FoldWhile, Itertools};

impl<'a, Cell> Command for Grid<'a, Cell>
where
    Cell: std::fmt::Display,
{
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        self.parts
            .split()
            .map(|cell| Print(cell).write_ansi(out))
            .fold_while(Ok(()), |_, next| match next {
                Err(error) => FoldWhile::Done(Err(error)),
                _ => FoldWhile::Continue(Ok(())),
            })
            .into_inner()
    }
}

//impl<'a, V> Command for V
//where
//V: View<'a, Parts = Graphemes<'a>>,
//{
//fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
//self.split()
//.map(|cell| Print(cell).write_ansi(out))
//.fold_while(Ok(()), |_, next| match next {
//Err(error) => FoldWhile::Done(Err(error)),
//_ => FoldWhile::Continue(Ok(())),
//})
//.into_inner()
//}
//}

//impl<'a, B> Command for Cells<'a, B>
//where
//B: UnicodeSegmentation,
//{
//fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
//self.buffer
//.graphemes(true)
//.map(|cell| Print(cell).write_ansi(out))
//.fold_while(Ok(()), |_, next| match next {
//Err(error) => FoldWhile::Done(Err(error)),
//_ => FoldWhile::Continue(Ok(())),
//})
//.into_inner()
//}
//}

//use unicode_segmentation::Graphemes;

//impl<'a, T> View<'a> for &'a T
//where
//T: UnicodeSegmentation,
//{
//type Items = T;

//fn cells(&'a self) -> Cells<'a, Self::Items> {
////let g = self.graphemes(true);
//Cells { buffer: self }
//}
//}

//impl<'a> Screen<'a> for &'a str {}

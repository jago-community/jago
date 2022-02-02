pub trait View<'a> {
    type Segment;
    type Segments: Iterator;

    fn segments(&'a self) -> Self::Segments;
}

use unicode_segmentation::{Graphemes, UnicodeSegmentation};

impl<'a> View<'a> for &'a str {
    type Segment = &'a str;
    type Segments = Graphemes<'a>;

    fn segments(&self) -> Self::Segments {
        self.graphemes(true)
    }
}

use crossterm::Command;

//impl<'a, V> Command for V
//where
//V: View<'a, Segment = &'a str, Segments = Graphemes<'a>>,
//{
//fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
//Ok(())
//}
//}

use crate::handle::Handle;

pub trait Screen<'a>: View<'a> + Handle {}

impl<'a> Screen<'a> for &'a str {}

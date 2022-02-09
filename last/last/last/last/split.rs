use unicode_segmentation::{UWordBounds, UnicodeSegmentation};

//pub struct Lines<'a> {
//buffer: Sequence<&'a str, UWordBounds<'a>>,
//}

//impl<'a> From<&'a str> for Lines<'a> {
//fn from(buffer: &'a str) -> Self {
//Self {
//buffer: Sequence(Box::new(buffer.split_word_bounds())),
//}
//}
//}

//impl<'a> Splitter<'a> for Lines<'a> {
//fn before(&mut self) -> Sequence<'a> {
//Box::new(self.buffer.split_word_bounds())
//}

//fn after(&mut self) -> Sequence {
//Box::new(self.buffer.split_word_bounds())
//}
//}

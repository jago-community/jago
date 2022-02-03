pub trait Parser: Iterator {
    type Part: IntoIterator<Item = Self::Item>;

    fn sep(&self) -> <<Self as Parser>::Part as IntoIterator>::IntoIter;
}

#[test]
fn test_split() {
    use itertools::assert_equal;

    let input = vec!["Hello, world!"];

    let want = vec![
        "H", "e", "l", "l", "o", ",", " ", "w", "o", "r", "l", "d", "!",
    ];

    assert_equal(split(input), want);
}

use unicode_segmentation::UnicodeSegmentation;

fn split<'a>(input: impl IntoIterator<Item = &'a str>) -> impl Iterator<Item = &'a str> {
    input.into_iter().flat_map(|item| item.graphemes(true))
}

pub trait Split: IntoIterator {
    type Sequence: IntoIterator<Item = Self::Item>;

    fn sep(&self) -> <<Self as Split>::Sequence as IntoIterator>::IntoIter;
}

pub struct Splitter<A, B> {
    a: A,
    b: std::marker::PhantomData<B>,
}

impl<'a, A> Split for Splitter<A, &'a str>
where
    A: IntoIterator<Item = &'a str>,
{
    fn sep(&self) -> Self::Sequence {
        split(self)
    }
}

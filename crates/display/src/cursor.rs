#[derive(Debug, PartialEq)]
pub struct Cursor {
    position: usize,
    chunk: usize,
    offset: usize,
}

impl From<(usize, usize, usize)> for Cursor {
    fn from((x, y, z): (usize, usize, usize)) -> Self {
        Self {
            position: z,
            chunk: y,
            offset: x,
        }
    }
}

impl Cursor {
    fn current<'a>(&self, buffer: &'a str) -> &'a str {
        &buffer[self.position..].graphemes(true).next().unwrap_or("")
    }

    fn forward(&self, other: Cursor) -> Self {
        Self {
            position: self.position + other.position,
            chunk: self.chunk + other.chunk,
            offset: other.offset,
        }
    }

    fn backward(&self, other: Cursor) -> Self {
        Self {
            position: self.position - other.position,
            chunk: self.chunk - other.chunk,
            offset: other.offset,
        }
    }
}

use unicode_segmentation::UnicodeSegmentation;

fn forward_graphemes(buffer: &str, count: usize) -> Cursor {
    let (mut position, mut chunk, mut offset) = (0, 0, 0);

    let graphemes = buffer.graphemes(true);

    unimplemented!()
}

#[test]
fn test_forward_graphemes() {
    let buffer = include_str!("../../../jago");

    let tests = vec![
        ((0, 0, 0), 1, (1, 0, 1), " "),
        ((1, 0, 1), 1, (2, 0, 2), "J"),
        ((2, 0, 2), 1, (3, 0, 3), "a"),
        ((3, 0, 3), 1, (4, 0, 4), "g"),
        ((4, 0, 4), 1, (5, 0, 5), "o"),
        ((5, 0, 5), 1, (0, 1, 7), "\n"),
        ((0, 0, 0), 1, (1, 0, 1), " "),
        ((1, 0, 1), 2, (3, 0, 3), "a"),
        ((3, 0, 3), 3, (0, 1, 7), "\n"),
        ((0, 1, 7), 4, (3, 2, 11), "C"),
        ((3, 2, 10), 5, (8, 2, 15), "e"),
    ];

    for (from, steps, to, want) in tests {
        let start: Cursor = from.into();
        let difference = forward_graphemes(&buffer[start.position..], steps);
        let got = start.forward(difference);

        assert_eq!(
            got,
            to.into(),
            "{:?} + {} = got {:?} want {:?} {:?}",
            start,
            steps,
            got,
            to,
            want
        );

        let got = start.current(buffer);

        assert_eq!(
            got, want,
            "{:?} + {} = got {:?} want {:?}",
            start, steps, got, want
        );
    }
}

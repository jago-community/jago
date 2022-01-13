#[derive(Default)]
pub struct Grid<'a> {
    bytes: &'a [u8],
    front: Cell<'a>,
    back: Cell<'a>,
}

impl<'a> From<&'a [u8]> for Grid<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            ..Default::default()
        }
    }
}

use unicode_segmentation::UnicodeSegmentation;

impl<'a> Grid<'a> {
    fn rest(&'a self, index: usize) -> &'a str {
        unsafe { std::str::from_utf8_unchecked(&self.bytes[index..]) }
    }

    fn after(&'a self, cell: &'a Cell) -> Option<&'a str> {
        self.bytes
            .get(cell.index + 1..)
            .map(|slice| unsafe { std::str::from_utf8_unchecked(slice) })
    }

    fn grapheme(&'a self, cell: &'a Cell<'a>) -> Option<&'a str> {
        self.rest(cell.index).graphemes(true).next()
    }
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Cell<'a> {
    index: usize,
    x: usize,
    y: usize,
    lifetime: std::marker::PhantomData<&'a bool>,
}

impl Cell<'_> {
    fn index_after(&self) -> usize {
        self.index + 1
    }
}

impl From<(usize, (usize, usize))> for Cell<'_> {
    fn from((index, (x, y)): (usize, (usize, usize))) -> Self {
        Self {
            index,
            x,
            y,
            lifetime: std::marker::PhantomData,
        }
    }
}

use itertools::{FoldWhile, Itertools};

impl<'a> Grid<'a> {
    fn cells_after(&self) -> impl Iterator<Item = Cell<'a>> {
        self.after(&self.front)
            .into_iter()
            .flat_map(|after| after.grapheme_indices(true))
            .batching(|it| match dbg!(it.next()) {
                Some((_, "\n")) => it.next().map(|(index, _)| (index, 0, 1)),
                Some((index, grapheme)) => Some((index, grapheme.len(), 0)),
                None => None,
            })
            // .scan and map
            .scan(self.front.clone(), |mut cell, (index, x, dy)| {
                // ...
                *cell = Cell::from((
                    dbg!(index + self.front.index_after()),
                    (dbg!(x), self.front.y + dbg!(dy)),
                ));

                Some(cell)
            })
            .map(|cell| *cell)
            //.map(|(index, x, dy)| {
            //Cell::from((
            //dbg!(index + self.front.index_after()),
            //(dbg!(x), self.front.y + dbg!(dy)),
            //))
            //})
            .inspect(|a| {
                dbg!(a);
            })
    }
}

#[test]
fn cells() {
    let bytes = include_bytes!("../poems/chris-abani/the-new-religion");

    let grid = Grid::from(bytes.as_ref());

    let mut cells = grid.cells_after();

    assert_eq!(cells.next(), Some((1, (1, 0)).into()));
    assert_eq!(grid.grapheme(&Cell::from((1, (1, 0)))), Some("h"));

    let mut cells = cells.skip(13);

    let next = cells.next().unwrap();

    assert_eq!(grid.grapheme(&next), Some("n"));

    let next = cells.next().unwrap();

    assert_eq!(next, (17, (0, 1)).into());
    assert_eq!(grid.grapheme(&next), Some("\n"));

    let next = cells.next().unwrap();

    assert_eq!(next, (18, (0, 2)).into());
    assert_eq!(grid.grapheme(&next), Some("T"));

    let next = cells.next().unwrap();

    assert_eq!(next, (19, (1, 2)).into());
    assert_eq!(grid.grapheme(&next), Some("h"));
}

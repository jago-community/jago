#[derive(Default)]
pub struct Buffer<'a> {
    bytes: &'a [u8],
    cursor: Cell,
}

impl<'a> From<&'a [u8]> for Buffer<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            cursor: Cell {
                position: 0,
                coordinates: Some((0, 0)),
            },
        }
    }
}

use unicode_segmentation::UnicodeSegmentation;

impl<'a> Buffer<'a> {
    pub fn grapheme(&self, index: usize) -> Option<&'a str> {
        unsafe { std::str::from_utf8_unchecked(&self.bytes[index..]) }
            .graphemes(true)
            .next()
    }

    pub fn current(&self) -> Option<&'a str> {
        self.grapheme(self.cursor.position)
    }

    fn cells(&self) -> Cells<'a> {
        Cells::new(self.bytes)
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Cell {
    position: usize,
    coordinates: Option<(usize, usize)>,
}

impl Cell {
    fn position(&self) -> usize {
        self.position
    }

    fn coordinates(&self) -> Option<(usize, usize)> {
        self.coordinates
    }
}

#[cfg(test)]
impl<'a> From<&'a (usize, (usize, usize))> for Cell {
    fn from((position, (x, y)): &'a (usize, (usize, usize))) -> Self {
        Self {
            position: *position,
            coordinates: Some((*x, *y)),
        }
    }
}

#[derive(Default)]
pub struct Cells<'a> {
    bytes: &'a [u8],
    front: Cell,
    back: Cell,
}

impl<'a> Cells<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            front: Cell {
                position: 0,
                coordinates: Some((0, 0)),
            },
            back: Cell {
                position: 0,
                coordinates: None,
            },
        }
    }
}

impl<'a> Iterator for Cells<'a> {
    type Item = Cell;

    fn next(&mut self) -> Option<Self::Item> {
        let bytes = self.bytes.get(self.front.position..)?;

        let mut graphemes = unsafe { std::str::from_utf8_unchecked(bytes) }.grapheme_indices(true);

        let (_, grapheme) = graphemes.next()?;

        self.front = Cell {
            position: self.front.position + 1,
            coordinates: self.front.coordinates.map(|(x, y)| match grapheme {
                "\n" => (0, y + 1),
                _ => (x + grapheme.len(), y),
            }),
        };

        if Some("\n") == Buffer::from(self.bytes).grapheme(self.front.position) {
            self.front = Cell {
                position: self.front.position + 1,
                coordinates: self.front.coordinates.map(|(_, y)| (0, y + 1)),
            };
        }

        Some(self.front.clone())
    }
}

#[test]
fn cells() {
    let bytes = include_bytes!("../poems/chris-abani/the-new-religion");

    let buffer = Buffer::from(&bytes[..]);
    let mut cells = Cells::new(bytes);

    assert_eq!(buffer.grapheme(1), Some("h"));
    assert_eq!(cells.next(), Some(Cell::from(&(1, (1, 0)))));
    assert_eq!(cells.next(), Some(Cell::from(&(2, (2, 0)))));

    let mut cells = cells.skip(12);

    let next = cells.next().unwrap();

    assert_eq!(next, Cell::from(&(15, (15, 0))));
    assert_eq!(buffer.grapheme(next.position), Some("n"));

    let next = cells.next().unwrap();

    assert_eq!(next, Cell::from(&(17, (0, 1))));
    assert_eq!(buffer.grapheme(17), Some("\n"));

    let next = cells.next().unwrap();

    assert_eq!(next, Cell::from(&(18, (0, 2))));
    assert_eq!(buffer.grapheme(18), Some("T"));

    let next = cells.next().unwrap();

    assert_eq!(next, Cell::from(&(19, (1, 2))));
    assert_eq!(buffer.grapheme(19), Some("h"));
}

use crossterm::{
    cursor::{MoveTo, MoveToColumn},
    style::{Color, Print, SetForegroundColor},
    terminal::{Clear, ClearType},
    Command,
};

use crate::color::ColorPicker;

impl<'a> Command for Buffer<'a> {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        Clear(ClearType::All).write_ansi(out)?;
        MoveTo(0, 0).write_ansi(out)?;

        let mut cells = self.cells();

        let mut color_picker = ColorPicker::new();

        while let Some(cell) = cells.next() {
            SetForegroundColor(color_picker.pick()).write_ansi(out)?;
            if let Some((x, y)) = cell.coordinates() {
                MoveTo(x as u16, y as u16).write_ansi(out)?;
            }
            if let Some(grapheme) = self.grapheme(cell.position()) {
                Print(grapheme).write_ansi(out)?;
            }
        }

        SetForegroundColor(Color::Green).write_ansi(out)?;
        Print(format!("\n{:?} {:?}", self.current(), self.cursor)).write_ansi(out)?;

        if let Some((x, y)) = self.cursor.coordinates() {
            MoveTo(x as u16, y as u16).write_ansi(out)?;
        }

        Ok(())
    }
}

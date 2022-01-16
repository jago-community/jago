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
        Cells {
            bytes: self.bytes,
            front: Cell {
                position: 0,
                coordinates: None,
            },
            ..Default::default()
        }
    }

    fn cells_after(&self) -> Cells<'a> {
        Cells {
            bytes: self.bytes,
            front: self.cursor.clone(),
            ..Default::default()
        }
    }

    fn cells_before(&self) -> Cells<'a> {
        Cells {
            bytes: self.bytes,
            back: Cell {
                position: self.bytes.len() - self.cursor.position,
                coordinates: None,
            },
            ..Default::default()
        }
    }
}

use crossterm::event::{Event, KeyCode, KeyEvent};

impl<'a> Buffer<'a> {
    pub fn handle(&mut self, event: &Event) {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                ..
            }) => {
                if let Some(next) = self.cells_after().next() {
                    self.cursor = next;
                }
            }
            _ => {}
        }
    }
}

use crossterm::{
    cursor::MoveTo,
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
                // TODO: only do this for the one after a new line.
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

#[test]
#[ignore]
fn cells() {
    let bytes = include_bytes!("../poems/chris-abani/the-new-religion");

    let mut buffer = Buffer::from(&bytes[..]);
    let mut cells = buffer.cells();

    assert_eq!(buffer.grapheme(0), Some("T"));
    assert_eq!(cells.next(), Some(Cell::from(&(0, (0, 0)))));

    let mut cells = buffer.cells_after();

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

    buffer.cursor = Cell::from(&(19, (1, 2)));

    let mut cells = buffer.cells_before();

    let next = cells.next_back().unwrap();

    assert_eq!(next, Cell::from(&(18, (0, 2))));
    assert_eq!(buffer.grapheme(18), Some("T"));

    let next = cells.next_back().unwrap();

    assert_eq!(next, Cell::from(&(17, (0, 1))));
    assert_eq!(buffer.grapheme(17), Some("\n"));

    let next = cells.next_back().unwrap();

    assert_eq!(next, Cell::from(&(15, (15, 0))));
    assert_eq!(buffer.grapheme(next.position), Some("n"));

    let next = cells.next_back().unwrap();

    assert_eq!(next, Cell::from(&(14, (14, 0))));
    assert_eq!(buffer.grapheme(next.position), Some("n"));

    let mut cells = cells.rev().skip(12);

    assert_eq!(next, Cell::from(&(1, (1, 0))));
    assert_eq!(buffer.grapheme(next.position), Some("h"));

    let next = cells.next().unwrap();

    assert_eq!(next, Cell::from(&(0, (0, 0))));
    assert_eq!(buffer.grapheme(0), Some("T"));

    assert!(cells.next().is_none());
}

impl<'a> Iterator for Cells<'a> {
    type Item = Cell;

    fn next(&mut self) -> Option<Self::Item> {
        if self.front.coordinates.is_none() {
            self.front.coordinates = Some((0, 0));

            return Some(self.front.clone());
        }

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

impl<'a> DoubleEndedIterator for Cells<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        // if self.back.coordinates.is_none() {
        //   self.back.coordinates = Some((0, 0));

        //   return Some(self.back.clone());
        // }

        let bytes = self.bytes.get(..self.bytes.len() - self.back.position)?;

        let mut graphemes = unsafe { std::str::from_utf8_unchecked(bytes) }
            .grapheme_indices(true)
            .rev();

        let (_, grapheme) = graphemes.next()?;

        self.back = Cell {
            position: self.back.position + 1,
            coordinates: self
                .back
                .coordinates
                .or(Some((0, 0)))
                .map(|(x, y)| match grapheme {
                    "\n" => (0, y + 1),
                    _ => (x + grapheme.len(), y),
                }),
        };

        if Some("\n") == Buffer::from(self.bytes).grapheme(self.back.position) {
            self.back = Cell {
                position: self.back.position + 1,
                coordinates: self.back.coordinates.map(|(_, y)| (0, y + 1)),
            };
        }

        Some(Cell {
            position: self.bytes.len() - dbg!(&self.back).position(),
            coordinates: None,
        })
    }
}

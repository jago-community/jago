#[derive(Default)]
pub struct Slice<'a> {
    bytes: &'a [u8],
    cursor: (usize, (usize, usize)),
    buffer: String,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Reference<'a>(usize, (usize, usize), std::marker::PhantomData<&'a ()>);

#[test]
fn slice_graphemes() {
    let bytes = include_bytes!("../poems/eltheridge-knight/haiku/1");

    let wants = vec![
        ((0, (0, 0)), "E"),
        ((1, (1, 0)), "a"),
        ((2, (2, 0)), "s"),
        ((3, (3, 0)), "t"),
        ((4, (4, 0)), "e"),
        ((5, (5, 0)), "r"),
        ((6, (6, 0)), "n"),
        ((7, (7, 0)), " "),
        ((8, (8, 0)), "g"),
        ((9, (9, 0)), "u"),
        ((10, (10, 0)), "a"),
        ((11, (11, 0)), "r"),
        ((12, (12, 0)), "d"),
        ((13, (13, 0)), " "),
        ((14, (14, 0)), "t"),
        ((15, (15, 0)), "o"),
        ((16, (16, 0)), "w"),
        ((17, (17, 0)), "e"),
        ((18, (18, 0)), "r"),
        // ((19, (19, 0)), "\n"),
        ((20, (0, 1)), "g"),
        ((21, (1, 1)), "l"),
    ]
    .into_iter()
    .map(|((index, coordinates), want)| ((index, coordinates), Some(want)));

    let slice = Slice {
        bytes,
        cursor: (0, (0, 0)),
        ..Default::default()
    };

    let gots = slice
        .grapheme_references()
        .take(21)
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants.clone(), gots);

    let gots = slice
        .grapheme_references_after(Reference::from((0, (0, 0))))
        .take(20)
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants.clone().skip(1), gots);

    let gots = slice
        .grapheme_references_after(Reference::from((1, (1, 0))))
        .take(19)
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants.clone().skip(2), gots);

    let gots = slice
        .grapheme_references_before((22, (2, 1)).into())
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants.rev(), gots);

    let bytes = include_bytes!("../poems/chris-abani/the-new-religion");

    let wants = vec![
        ((0, (0, 0)), "T"),
        ((1, (1, 0)), "h"),
        ((2, (2, 0)), "e"),
        ((3, (3, 0)), " "),
        ((4, (4, 0)), "N"),
        ((5, (5, 0)), "e"),
        ((6, (6, 0)), "w"),
        ((7, (7, 0)), " "),
        ((8, (8, 0)), "R"),
        ((9, (9, 0)), "e"),
        ((10, (10, 0)), "l"),
        ((11, (11, 0)), "i"),
        ((12, (12, 0)), "g"),
        ((13, (13, 0)), "i"),
        ((14, (14, 0)), "o"),
        ((15, (15, 0)), "n"),
        // ((16, (16, 0)), "\n"),
        ((17, (0, 1)), "\n"),
        ((18, (0, 2)), "T"),
        ((19, (1, 2)), "h"),
        ((20, (2, 2)), "e"),
    ]
    .into_iter()
    .map(|((index, coordinates), want)| ((index, coordinates), Some(want)));

    let slice = Slice {
        bytes,
        cursor: (0, (0, 0)),
        ..Default::default()
    };

    let gots = slice
        .grapheme_references()
        .take(20)
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants.clone(), gots);

    let gots = slice
        .grapheme_references_before((20, (2, 2)).into())
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants.clone().rev().skip(1), gots);

    let gots = slice
        .grapheme_references_after((17, (0, 1)).into())
        .take(3)
        .map(|reference| (reference.layout(), slice.get(reference)));

    dbg!("last");

    itertools::assert_equal(wants.clone().skip(17), gots);
}

impl<'a> From<&'a [u8]> for Slice<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            cursor: (0, (0, 0)),
            ..Default::default()
        }
    }
}

impl<'a> From<(usize, (usize, usize))> for Reference<'a> {
    fn from((index, coordinates): (usize, (usize, usize))) -> Self {
        Self(index, coordinates, Default::default())
    }
}

use itertools::Itertools;
use unicode_segmentation::UnicodeSegmentation;

impl<'a> Slice<'a> {
    fn cast_str(&'a self) -> &'a str {
        unsafe { std::str::from_utf8_unchecked(self.bytes) }
    }

    fn get(&'a self, reference: Reference) -> Option<&'a str> {
        self.cast_str()
            .get(reference.index()..)
            .iter()
            .flat_map(|slice| slice.graphemes(true))
            .next()
    }

    pub fn grapheme_references(&'a self) -> impl Iterator<Item = Reference> {
        self.grapheme_reference_span((0, (0, 0)).into(), self.bytes.len())
            .into_iter()
    }

    pub fn grapheme_reference_span(
        &'a self,
        reference: Reference,
        span: usize,
    ) -> impl Iterator<Item = Reference> {
        self.bytes
            .get(reference.index()..reference.index() + span)
            .into_iter()
            .map(|slice| unsafe { std::str::from_utf8_unchecked(slice) })
            .flat_map(|slice| slice.graphemes(true))
            .scan(reference.layout(), |(position, coordinates), grapheme| {
                let current = Some(Reference::from((*position, *coordinates)));

                *position += grapheme.len();

                match grapheme {
                    "\n" => {
                        coordinates.0 = 0;
                        coordinates.1 += 1;
                    }
                    _ => {
                        coordinates.0 += 1;
                    }
                };

                current
            })
            .enumerate()
            .batching(|it| {
                let (index, next) = it.next()?;

                match (index > 0, self.bytes.get(next.index())) {
                    (true, Some(b'\n')) => it.next().map(|(_, next)| next),
                    _ => Some(next),
                }
            })
    }

    pub fn grapheme_references_after(
        &self,
        reference: Reference,
    ) -> impl Iterator<Item = Reference> {
        self.grapheme_reference_span(reference.clone(), self.bytes.len() - reference.index())
            .skip(1)
    }

    pub fn grapheme_references_before(
        &'a self,
        reference: Reference,
    ) -> impl Iterator<Item = Reference> {
        let (index, coordinates) = reference.layout();

        self.bytes
            .get(..=index)
            .into_iter()
            .map(|slice| unsafe { std::str::from_utf8_unchecked(slice) })
            .flat_map(|slice| slice.graphemes(true))
            .rev()
            .scan((index, coordinates), |(position, coordinates), _| {
                *position = position.checked_sub(1)?;

                if let Some(next) = coordinates.0.checked_sub(1) {
                    coordinates.0 = next;
                } else {
                    coordinates.1 = coordinates.1.checked_sub(1)?;

                    coordinates.0 = *position
                        - self
                            .cast_str()
                            .get(..*position)
                            .into_iter()
                            .flat_map(|as_str| as_str.grapheme_indices(true))
                            .rev()
                            .find_map(|(index, grapheme)| {
                                if index == 0 {
                                    Some(0)
                                } else if grapheme == "\n" {
                                    Some(index + 1)
                                } else {
                                    None
                                }
                            })?;
                }

                Some(Reference::from((*position, *coordinates)))
            })
            .batching(|it| {
                let next = it.next()?;

                match self.bytes.get(next.index()) {
                    Some(b'\n') if next.coordinates().0 > 0 => it.next(),
                    _ => Some(next),
                }
            })
    }
}

impl<'a> Reference<'a> {
    fn index(&self) -> usize {
        self.0
    }

    fn coordinates(&self) -> (usize, usize) {
        self.1
    }

    fn layout(&self) -> (usize, (usize, usize)) {
        (self.0, self.1)
    }
}

use crossterm::event::{Event, KeyCode, KeyEvent};

impl<'a> Slice<'a> {
    pub fn handle(&mut self, event: &Event) {
        let mut next = self.cursor.clone();

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                ..
            }) => {
                let mut references = self.grapheme_references_after(self.cursor.into());

                if let Some(next_ref) = references.next() {
                    next = (next_ref.index(), next_ref.coordinates());
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('h'),
                ..
            }) => {
                let mut references = self.grapheme_references_before(self.cursor.into());

                if let Some(next_ref) = references.next() {
                    next = (next_ref.index(), next_ref.coordinates());
                }
            }
            _ => {}
        }

        self.cursor = next;
    }
}

use crossterm::{
    cursor::MoveTo,
    style::{Color, Print, SetForegroundColor},
    terminal::{Clear, ClearType},
    Command,
};

use crate::color::ColorPicker;

impl<'a> Command for Slice<'a> {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        Clear(ClearType::All).write_ansi(out)?;
        MoveTo(0, 0).write_ansi(out)?;

        let mut references = self.grapheme_references();

        let mut color_picker = ColorPicker::new();

        while let Some(reference) = references.next() {
            SetForegroundColor(color_picker.pick()).write_ansi(out)?;

            let (x, y) = reference.coordinates();
            // TODO: only do this for the one after a new line.
            MoveTo(x as u16, y as u16).write_ansi(out)?;

            if let Some(grapheme) = self.get(reference) {
                Print(grapheme).write_ansi(out)?;
            }
        }

        SetForegroundColor(Color::Green).write_ansi(out)?;
        Print(format!(
            "\n{:?} {:?}\n\n{:?}",
            Reference::from(self.cursor).layout(),
            self.get(Reference::from(self.cursor)),
            self.buffer
        ))
        .write_ansi(out)?;

        let (x, y) = Reference::from(self.cursor).coordinates();

        MoveTo(x as u16, y as u16).write_ansi(out)?;

        Ok(())
    }
}

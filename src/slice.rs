#[derive(Default)]
pub struct Slice<'a> {
    bytes: &'a [u8],
    cursor: (usize, Option<(usize, usize)>),
    buffer: String,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Reference(usize, Option<(usize, usize)>);

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
        ((19, (19, 0)), "\n"),
        ((20, (0, 1)), "g"),
        ((21, (1, 1)), "l"),
    ]
    .into_iter()
    .map(|((index, coordinates), want)| ((index, Some(coordinates)), Some(want)));

    let slice = Slice {
        bytes,
        cursor: (0, None),
        ..Default::default()
    };

    let gots = slice
        .grapheme_references()
        .take(22)
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants.clone(), gots);

    let gots = slice
        .grapheme_references_after(Reference::from((0, Some((0, 0)))))
        .take(21)
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants.clone(), gots);

    let wants = wants.rev();

    let gots = slice
        .grapheme_references_before((22, Some((2, 1))).into())
        .map(|reference| (reference.layout(), slice.get(reference)));

    itertools::assert_equal(wants, gots);
}

impl<'a> From<&'a [u8]> for Slice<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            cursor: (0, Some((0, 0))),
            ..Default::default()
        }
    }
}

impl From<(usize, Option<(usize, usize)>)> for Reference {
    fn from((index, coordinates): (usize, Option<(usize, usize)>)) -> Self {
        Self(index, coordinates)
    }
}

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
        self.cast_str()
            .get(..)
            .into_iter()
            .flat_map(|slice| slice.graphemes(true))
            .scan((0, (0, 0)), |(position, coordinates), grapheme| {
                let current = Some(Reference::from((*position, Some(*coordinates))));

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
    }

    pub fn grapheme_references_after1(
        &'a self,
        reference: &'a Reference,
    ) -> impl Iterator<Item = &'a Reference> {
        let (position, coordinates) = reference.layout();

        self.cast_str()
            .get(reference.index() + 1..)
            .into_iter()
            .flat_map(|slice| slice.graphemes(true))
            .scan(
                (position, coordinates.unwrap_or((0, 0))),
                |(position, coordinates), grapheme| {
                    let current = Some(Reference::from((*position, Some(*coordinates))));

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
                },
            )
    }

    pub fn grapheme_references_after(
        &'a self,
        reference: Reference,
    ) -> impl Iterator<Item = Reference> {
        let (position, coordinates) = reference.layout();

        self.cast_str()
            .get(reference.index() + 1..)
            .into_iter()
            .flat_map(|as_str| as_str.graphemes(true))
            .scan(
                (position, coordinates.unwrap_or((0, 0))),
                |(position, coordinates), grapheme| {
                    let current = Some(Reference::from((*position, Some(*coordinates))));

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
                },
            )
    }

    pub fn grapheme_references_before(
        &'a self,
        reference: Reference,
    ) -> impl Iterator<Item = Reference> {
        let (index, coordinates) = reference.layout();

        self.cast_str()
            .get(..=reference.index())
            .into_iter()
            .flat_map(|as_str| as_str.graphemes(true))
            .rev()
            .scan(
                (index, coordinates.unwrap()),
                |(position, coordinates), _| {
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
                                .inspect(|a| {
                                    dbg!(a);
                                })
                                .flat_map(|as_str| as_str.grapheme_indices(true))
                                .rev()
                                .find_map(|(index, grapheme)| {
                                    if grapheme == "\n" || index == 0 {
                                        Some(index)
                                    } else {
                                        None
                                    }
                                })?;
                    }

                    Some(Reference::from((*position, Some(*coordinates))))
                },
            )
    }
}

impl Reference {
    fn index(&self) -> usize {
        self.0
    }

    fn coordinates(&self) -> Option<(usize, usize)> {
        self.1
    }

    fn layout(&self) -> (usize, Option<(usize, usize)>) {
        (self.0, self.1)
    }
}

use crossterm::event::{Event, KeyCode, KeyEvent};

impl<'a> Slice<'a> {
    pub fn handle(&mut self, event: &Event) {
        let mut next = self.cursor.clone();

        let mut next_buffer = format!("{:?}", event);

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                ..
            }) => {
                let mut references = self.grapheme_references_after(Reference::from(self.cursor));

                if let Some(next_ref) = references.next() {
                    next = (next_ref.index(), next_ref.coordinates());
                    next_buffer.push_str(&format!("{:?}", next_ref));
                }
            }
            _ => {}
        }

        self.cursor = next;
        self.buffer = next_buffer;
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

            if let Some((x, y)) = reference.coordinates() {
                // TODO: only do this for the one after a new line.
                MoveTo(x as u16, y as u16).write_ansi(out)?;
            }

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

        if let Some((x, y)) = Reference::from(self.cursor).coordinates() {
            MoveTo(x as u16, y as u16).write_ansi(out)?;
        }

        Ok(())
    }
}

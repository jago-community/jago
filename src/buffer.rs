#[derive(Default, Debug)]
pub struct Buffer {
    bytes: Vec<u8>,
    cursor: Cursor,
    sequence: Vec<char>,
    mode: Mode,
    dimensions: Option<(u16, u16)>,
}

pub type Cursor = (usize, (usize, usize));

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

impl<'a> From<&'a [u8]> for Buffer {
    fn from(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes.into(),
            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq)]
enum Mode {
    Cursor,
    Edit,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Cursor
    }
}

use crate::color::Color;

enum TextStyle {
    Underline,
}

pub enum Block {
    NewLine,
    Text(String),
    Active,
    Inactive,
    Color(Option<Color>),
    Group(Vec<Block>),
    Cursor(Cursor),
    Empty,
}

use crossterm::{
    cursor::{MoveTo, MoveToColumn},
    style::{Attribute, SetAttributes},
    style::{Print, ResetColor, SetForegroundColor},
    Command,
};

impl Command for Block {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            Block::NewLine => MoveToColumn(0).write_ansi(out),
            Block::Text(text) => Print(text).write_ansi(out),
            Block::Color(Some(color)) => SetForegroundColor(*color).write_ansi(out),
            Block::Color(None) => ResetColor.write_ansi(out),
            Block::Cursor((_, (x, y))) => MoveTo(*x as u16, *y as u16).write_ansi(out),
            Block::Active => SetAttributes(From::from(
                [Attribute::Underlined, Attribute::Bold].as_ref(),
            ))
            .write_ansi(out),
            Block::Inactive => SetAttributes(From::from(Attribute::Reset)).write_ansi(out),
            Block::Group(slice) => slice
                .iter()
                .map(|block| block.write_ansi(out))
                .find(|result| result.is_err())
                .unwrap_or(Ok(())),
            Block::Empty => Ok(()),
        }
    }
}

use crate::color::ColorPicker;

use unicode_segmentation::UnicodeSegmentation;

impl Buffer {
    pub fn write_terminal(&mut self) -> Result<Block, Error> {
        let mut color_picker = ColorPicker::new();

        let ys = self.dimensions.map(|(_, y)| y);

        let words = unsafe { std::str::from_utf8_unchecked(&self.bytes) }
            .split_word_bounds()
            .scan(0, |y, word| {
                if word == "\n" {
                    *y += 1;
                }

                Some((*y, word))
            })
            .take_while(|(line, _)| ys.map(|y| &(y - 3) > line).unwrap_or(false))
            .flat_map(|(_, word)| {
                [
                    Block::Color(Some(color_picker.pick())),
                    Block::Text(word.into()),
                    Block::Color(None),
                    if word == "\n" {
                        Block::NewLine
                    } else {
                        Block::Empty
                    },
                ]
                .into_iter()
            });

        Ok(Block::Group(
            words
                .chain([Block::Cursor(self.cursor)].into_iter())
                .collect(),
        ))
    }
}

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::slice::Slice;

impl Buffer {
    pub fn as_slice(&self) -> Slice {
        Slice::from(self.bytes.as_slice())
    }

    pub fn handle(&mut self, event: &Event) -> Result<(), Error> {
        let mut stop = false;

        let mut next_cursor = self.cursor.clone();

        match event {
            Event::Resize(x, y) => {
                self.dimensions = Some((*x, *y));
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => {
                stop = true;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('h'),
                ..
            }) if self.mode == Mode::Cursor => {
                let factor = self.consume_factor();

                let slice = self.as_slice();

                let mut references = slice.graphemes_before(self.cursor.into()).skip(factor - 1);

                if let Some(next_ref) = references.next() {
                    next_cursor = (next_ref.index(), next_ref.coordinates());
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                ..
            }) if self.mode == Mode::Cursor => {
                let factor = self.consume_factor();

                let slice = self.as_slice();

                let mut references = slice
                    .closest_x_in_y_after(self.cursor.into())
                    .skip(factor - 1);

                if let Some(next_ref) = references.next() {
                    next_cursor = (next_ref.index(), next_ref.coordinates());
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                ..
            }) if self.mode == Mode::Cursor => {
                let factor = self.consume_factor();

                let slice = self.as_slice();

                let mut references = slice
                    .closest_x_in_y_before(self.cursor.into())
                    .skip(factor - 1);

                if let Some(next_ref) = references.next() {
                    next_cursor = (next_ref.index(), next_ref.coordinates());
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                ..
            }) if self.mode == Mode::Cursor => {
                let factor = self.consume_factor();

                let slice = self.as_slice();

                let mut references = slice.graphemes_after(self.cursor.into()).skip(factor - 1);

                if let Some(next_ref) = references.next() {
                    next_cursor = (next_ref.index(), next_ref.coordinates());
                }
            }
            _ => {}
        };

        self.cursor = next_cursor;

        if stop {
            Err(Error::Incomplete)
        } else {
            Ok(())
        }
    }
}

fn factor(sequence: &[char]) -> (usize, usize) {
    let got = sequence
        .iter()
        .enumerate()
        .map(|(index, maybe_digit)| (index + 1, maybe_digit.to_digit(10)))
        .take_while(|(_, result)| result.is_some())
        .map(|(index, result)| (index, result.unwrap()))
        .fold((0, 0), |(_, factor), (index, digit)| {
            (index, factor * 10 + digit)
        });

    (got.0 as usize, if got.0 == 0 { 1 } else { got.1 as usize })
}

impl Buffer {
    fn consume_factor(&mut self) -> usize {
        let got = factor(&self.sequence);

        self.sequence = self.sequence.drain(got.0..).collect();

        got.1
    }
}

#[test]
fn test_factor() {
    let sequences = vec![
        (vec!['1', '0', '2'], 3, 102),
        (vec!['2'], 1, 2),
        (vec![], 0, 1),
        (vec!['b'], 0, 1),
    ];

    for (sequence, want_took, want_factor) in sequences {
        let mut buffer = Buffer {
            sequence: sequence.clone(),
            ..Default::default()
        };

        assert_eq!(buffer.consume_factor(), want_factor);

        assert_eq!(&buffer.sequence[..], &sequence[want_took..]);
    }
}

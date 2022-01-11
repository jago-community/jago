pub struct Buffer<'a> {
    bytes: &'a [u8],
    cursor: (usize, (usize, usize)),
    sequence: Vec<char>,
}

impl<'a> Buffer<'a> {
    pub fn new(bytes: &'a [u8]) -> Buffer<'a> {
        Buffer {
            bytes,
            cursor: (0, (0, 0)),
            sequence: vec![],
        }
    }

    fn as_str(&self) -> &'a str {
        unsafe { std::str::from_utf8_unchecked(self.bytes) }
    }
}

use crossterm::{
    cursor::{MoveTo, MoveToColumn},
    style::{Color, Print, SetForegroundColor},
    terminal::{Clear, ClearType},
    Command,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::color::ColorPicker;

impl<'a> Command for Buffer<'a> {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        Clear(ClearType::All).write_ansi(out)?;
        MoveTo(0, 0).write_ansi(out)?;

        let buffer = self.as_str();

        let mut color_picker = ColorPicker::new();

        let result = buffer
            .graphemes(true)
            .into_iter()
            .fold(Ok((0, 0)), |result, token| {
                if let Err(error) = SetForegroundColor(color_picker.pick()).write_ansi(out) {
                    return Err(error);
                }

                if let Err(error) = Print(token).write_ansi(out) {
                    return Err(error);
                }

                if token == "\n" {
                    if let Err(error) = MoveToColumn(0).write_ansi(out) {
                        return Err(error);
                    }
                }

                result
            });

        if let Err(error) = result {
            return Err(error);
        }

        SetForegroundColor(Color::Green).write_ansi(out)?;
        Print(format!(
            "\n{:?} {:?}",
            current_grapheme(buffer, self.cursor.0),
            self.cursor
        ))
        .write_ansi(out)?;

        MoveTo(self.cursor.1 .0 as u16, self.cursor.1 .1 as u16).write_ansi(out)?;

        Ok(())
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
                self.cursor = forward_graphemes(self.as_str(), self.cursor, self.factor());

                if self.sequence.len() > 0 {
                    self.sequence = vec![];
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(code),
                ..
            }) => {
                self.sequence.push(*code);
            }
            _ => {}
        };
    }
}

use itertools::{FoldWhile, Itertools};

impl<'a> Buffer<'a> {
    fn factor(&self) -> usize {
        self.sequence
            .iter()
            .fold_while(0, |scale, c| {
                if let Some(digit) = c.to_digit(10) {
                    FoldWhile::Continue(scale * 10 + digit)
                } else {
                    FoldWhile::Done(scale)
                }
            })
            .into_inner() as usize
    }
}

fn current_grapheme<'a>(buffer: &'a str, position: usize) -> Option<&'a str> {
    buffer[position..].graphemes(true).next()
}

fn forward_graphemes(
    buffer: &str,
    (start_position, (start_x, start_y)): (usize, (usize, usize)),
    count: usize,
) -> (usize, (usize, usize)) {
    buffer[start_position..]
        .grapheme_indices(true)
        .skip(1)
        // .batching if current/next == "\n" skip 1
        .fold_while(
            (start_position, (start_x, start_y)),
            |(position, (x, y)), (index, grapheme)| {
                let next_position = start_position + index;

                let next = match grapheme {
                    "\n" => (next_position + 1, (0, y + 1)),
                    _ => (next_position, (x + grapheme.len(), y)),
                };

                let saw = dbg!(dbg!(next).0 - dbg!(start_position) - dbg!(y) - dbg!(start_y));

                let wrap = if saw >= count {
                    FoldWhile::Done
                } else {
                    FoldWhile::Continue
                };

                wrap(next)
            },
        )
        .into_inner()
}

#[test]
fn test_forward_graphemes() {
    let buffer = include_str!("../poems/chris-abani/the-new-religion");

    let tests = vec![
        ((0, (0, 0)), 1, (1, (1, 0)), "h"),
        ((15, (15, 0)), 1, (17, (0, 1)), "\n"),
        ((0, (0, 0)), 2, (2, (2, 0)), "e"),
        ((15, (15, 0)), 3, (19, (1, 2)), "h"),
    ];

    for (start, step, want, grapheme) in tests {
        dbg!("start");
        let got = forward_graphemes(buffer, start, step);
        assert_eq!(
            got,
            want,
            "{:?} + {} = got {:?} {:?} want {:?} {:?}",
            start,
            step,
            got,
            current_grapheme(buffer, got.0),
            want,
            grapheme
        );
    }
}

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

/*
 The New Religion

 The body is a nation I have not known.

 T  0  0, 0
 h  1  1, 0
 e  2  2, 0
 <...>
 n 15 15, 0
\n 16 16, 0
\n 17  0, 1
 T 18  0, 1
 h 19  1, 1
 e 20  2, 1
*/
fn forward_graphemes(
    buffer: &str,
    (start_position, (start_x, start_y)): (usize, (usize, usize)),
    count: usize,
) -> (usize, (usize, usize)) {
    dbg!(start_position);

    buffer[start_position..]
        .grapheme_indices(true)
        //.skip(1)
        // .batching if current/next == "\n" skip 1
        .batching(|it| match it.next() {
            Some((_, "\n")) => it.next().map(|(index, _)| (index, (0, 1))),
            Some((index, grapheme)) => Some((index, (grapheme.len(), 0))),
            None => None,
        })
        // need to track diffs from batches
        // TODO: count blocks (from notebook).
        .fold_while(
            (start_position, (start_x, start_y)),
            |(index, (x, y)), (next_index, (dx, dy))| {
                dbg!(index, (dx, dy));

                unimplemented!()
            },
        )
        .into_inner()
}

fn forward_graphemes1(
    buffer: &str,
    (start_position, (start_x, start_y)): (usize, (usize, usize)),
    count: usize,
) -> (usize, (usize, usize)) {
    buffer[start_position..]
        .grapheme_indices(true)
        .skip(1)
        // .batching if current/next == "\n" skip 1
        .batching(|it| match dbg!(it.next()) {
            Some((_, "\n")) => dbg!(it.next()).map(|(index, grapheme)| (index, grapheme, true)),
            Some((index, grapheme)) => Some((index, grapheme, false)),
            None => None,
        })
        // need to track diffs from batches
        .fold_while(
            (start_position, (start_x, start_y)),
            |(_, (x, y)), (index, grapheme, start_line)| {
                //dbg!((start_position + index, index, grapheme));

                let next = if start_line {
                    (1 + start_position + index, (0, y + 1))
                } else {
                    (1 + start_position + index, (x + 1, y))
                };

                let wrap = if index + 1 >= count {
                    dbg!(1);

                    FoldWhile::Done
                } else {
                    dbg!(2);

                    FoldWhile::Continue
                };

                wrap(next)
            },
        )
        /*
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
        )*/
        .into_inner()
}

#[test]
#[ignore]
fn test_forward_graphemes() {
    let buffer = include_str!("../poems/chris-abani/the-new-religion");

    let tests = vec![
        ((0, (0, 0)), 1, (1, (1, 0)), "h"),
        ((15, (15, 0)), 1, (17, (0, 1)), "\n"),
        ((0, (0, 0)), 2, (2, (2, 0)), "e"),
        ((15, (15, 0)), 3, (19, (1, 2)), "h"),
    ];

    for (start, step, want, grapheme) in tests {
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

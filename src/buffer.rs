pub struct Buffer<'a> {
    bytes: &'a [u8],
    cursor: (usize, (usize, usize)),
}

impl<'a> Buffer<'a> {
    pub fn new(bytes: &'a [u8]) -> Buffer<'a> {
        Buffer {
            bytes,
            cursor: (0, (0, 0)),
        }
    }

    fn as_str(&self) -> &'a str {
        unsafe { std::str::from_utf8_unchecked(self.bytes) }
    }
}

use crossterm::{
    cursor::{position, MoveTo, MoveToColumn},
    style::{Print, SetForegroundColor},
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

        SetForegroundColor(color_picker.pick()).write_ansi(out)?;
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
                self.cursor = forward_graphemes(self.as_str(), self.cursor, 1);
            }
            _ => {}
        };
    }
}

fn current_grapheme<'a>(buffer: &'a str, position: usize) -> Option<&'a str> {
    buffer[position..].graphemes(true).next()
}

use itertools::{FoldWhile, Itertools};

fn forward_graphemes(
    buffer: &str,
    (start_position, (start_x, start_y)): (usize, (usize, usize)),
    count: usize,
) -> (usize, (usize, usize)) {
    buffer[start_position..]
        .graphemes(true)
        .fold_while(
            (start_position, (start_x, start_y)),
            |(position, (x, y)), grapheme| {
                let next = match dbg!(grapheme) {
                    "\n" => (position + 1, (0, y + 1)),
                    _ => (position + grapheme.len(), (x + grapheme.len(), y)),
                };

                let wrap = if current_grapheme(buffer, next.0) == Some("\n")
                    && next.0 - start_position == count
                {
                    FoldWhile::Continue
                } else if next.0 - start_position >= count {
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

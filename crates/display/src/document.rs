#[derive(Default)]
pub struct Document<'a> {
    source: &'a str,
    buffer: String,
    focus: (u16, u16),
    cursor: usize,
    perspective: Perspective,
}

enum Perspective {
    Before,
    After,
}

impl Default for Perspective {
    fn default() -> Self {
        Perspective::Before
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Machine {0}")]
    Machine(#[from] std::io::Error),
    #[error("String {0}")]
    String(#[from] std::string::FromUtf8Error),
    #[error("Incomplete")]
    Incomplete,
}

use crossterm::{
    cursor::{MoveDown, MoveLeft, MoveRight, MoveTo, MoveUp},
    event::{Event, KeyCode, KeyEvent},
    queue, Command,
};

use std::io::Write;

impl<'a> Document<'a> {
    pub fn new(source: &'a [u8]) -> Result<Self, Error> {
        Ok(Self {
            source: unsafe { std::str::from_utf8_unchecked(source) },
            buffer: String::from_utf8(source.into())?,
            ..Default::default()
        })
    }

    pub fn focus(&mut self, focus: (u16, u16)) {
        self.focus = focus;
    }

    // TODO: handle events (wraps to next line if past end of line)

    pub fn handle(&mut self, event: &Event, mut output: impl Write) -> Result<(), Error> {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('h'),
                ..
            }) => {
                if let Some(start) = self.cursor.checked_sub(1) {
                    if &self.source[start..self.cursor] == "\n" {
                        queue!(output, MoveTo(0, self.focus.1 - 1))?;
                    } else {
                        queue!(output, MoveLeft(1))?;
                    }

                    self.cursor = start;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                ..
            }) => {
                if let Some(stop) = self.cursor.checked_add(1) {
                    if &self.source[self.cursor..stop] == "\n" {
                        queue!(output, MoveTo(0, self.focus.1 + 1))?;
                    } else if self.source.len() > stop {
                        queue!(output, MoveRight(1))?;
                    }

                    self.cursor = stop;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                ..
            }) => {
                while Some("\n")
                    != self
                        .source
                        .get(self.cursor..self.cursor.checked_add(1).ok_or(Error::Incomplete)?)
                {
                    self.cursor += 1;
                }

                let mut dx = 0;

                for x in 0..self.focus.0 as usize {
                    if &self.source[self.cursor + dx..self.cursor + dx + 1] == "\n" {
                        break;
                    }

                    dx = x;
                }

                self.cursor += dx;

                //if &self.source[self.cursor..] == "\n" {
                queue!(output, MoveTo(self.focus.0 + dx as u16, self.focus.1 + 1))?;
                //if &self.source[self.cursor..] == "\n" {
                //queue!(output, MoveRight(1))?;
                //}

                //queue!(output, MoveDown(1))?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                ..
            }) => {
                let mut dx = 0;

                while Some("\n")
                    != self
                        .source
                        .get(self.cursor.checked_sub(1).ok_or(Error::Incomplete)?..self.cursor)
                {
                    self.cursor -= 1;

                    dx += 1;
                }

                queue!(output, MoveTo(dx as u16, self.focus.1 - 1))?;
            }
            _ => {}
        };

        Ok(())
    }
}

impl<'a> Document<'a> {
    fn color(&self, index: usize) -> u8 {
        (index % 230) as u8
    }
}

use unicode_segmentation::UnicodeSegmentation;

impl<'a> Command for Document<'a> {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        let (mut x, mut y) = (0, 0);

        for (index, grapheme) in self.source.grapheme_indices(true) {
            let focus = self.focus == (x, y);

            crossterm::style::SetForegroundColor(crossterm::style::Color::AnsiValue(
                self.color(index),
            ))
            .write_ansi(out)?;

            out.write_str(if focus && grapheme == " " {
                "_"
            } else {
                grapheme
            })?;

            if grapheme == "\n" {
                x = 0;
                y += 1;
            } else {
                x += grapheme.len() as u16;
            }
        }

        Ok(())
    }
}

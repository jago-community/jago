use std::borrow::Cow;

pub struct Grid<'a> {
    buffer: Cow<'a, str>,
    dimensions: (usize, usize),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
}

impl<'a> Grid<'a> {
    pub fn new(buffer: &'a str, (x, y): (usize, usize)) -> Self {
        Self {
            buffer: buffer.into(),
            dimensions: (x, y),
        }
    }
}

use textwrap::fill;

impl<'a> Grid<'a> {
    pub fn buffer(&self) -> String {
        fill(&self.buffer, self.dimensions.0 - 10)
    }
}

use crossterm::{
    style::{Print, SetForegroundColor},
    Command,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::color::ColorPicker;

impl<'a> Command for Grid<'a> {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        let mut color_picker = ColorPicker::new();

        self.buffer()
            .split_sentence_bounds()
            .scan(0usize, |mut y, part| {
                if part == "\n" {
                    *y += 1;
                }

                Some((*y, part))
            })
            .take_while(|(line, _)| line < &self.dimensions.1)
            .map(|(_, part)| {
                SetForegroundColor(color_picker.pick())
                    .write_ansi(out)
                    .and(Print(part).write_ansi(out))
            })
            .find(Result::is_err)
            .unwrap_or(Ok(()))
    }
}

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

impl<'a> Grid<'a> {
    pub fn handle(&mut self, event: &Event) -> Result<(), Error> {
        let mut stop = false;

        match event {
            Event::Resize(x, y) => {
                self.dimensions = (*x as usize, *y as usize);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => {
                stop = true;
            }
            _ => {}
        };

        if stop {
            Err(Error::Incomplete)
        } else {
            Ok(())
        }
    }
}

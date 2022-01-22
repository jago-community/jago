#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

use crate::color::Color;

pub type Cursor = (usize, (usize, usize));

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

pub struct Buffer {
    bytes: Vec<u8>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self { bytes: vec![] }
    }
}

impl<'a> From<&'a [u8]> for Buffer {
    fn from(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes.into(),
        }
    }
}

use crate::color::ColorPicker;

use unicode_segmentation::UnicodeSegmentation;

impl Buffer {
    pub fn write_terminal(&mut self) -> Result<Block, Error> {
        let mut color_picker = ColorPicker::new();

        let words = unsafe { std::str::from_utf8_unchecked(&self.bytes) }
            .split_word_bounds()
            .flat_map(|word| {
                [
                    Block::Color(Some(color_picker.pick())),
                    Block::Text(word.into()),
                    Block::Color(None),
                ]
                .into_iter()
            });

        Ok(Block::Group(vec![].into_iter().chain(words).collect()))
    }
}

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

impl Buffer {
    pub fn handle(&mut self, event: &Event) -> Result<(), ()> {
        let mut stop = false;

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => {
                stop = true;
            }
            _ => {}
        };

        if stop {
            Err(())
        } else {
            Ok(())
        }
    }
}

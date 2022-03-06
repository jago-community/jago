#[derive(PartialEq)]
pub enum Outcome {
    Continue,
    Done,
    Exit(Option<i32>),
}

use crossterm::event::Event;

pub trait Handler {
    fn handle(&mut self, event: &Event) -> Outcome;

    fn handle_common(&mut self, event: &Event) -> Outcome {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => Outcome::Exit(None),
            _ => Outcome::Continue,
        }
    }
}

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crossterm::Command;

pub trait Viewer {
    type View: Command;

    fn view(&self) -> Self::View;
}

impl<D: std::fmt::Display> Viewer for D {
    fn view(&self) -> Lense<'_> {
        Lense::Encoded(Box::new(self))
    }
}

use crate::color::Color;

enum TextStyle {
    Underline,
}

pub enum Lense<'a> {
    Group(Vec<Lense<'a>>),
    NewLine,
    Encoded(Box<dyn std::fmt::Display + 'a>),
    Active,
    Inactive,
    Color(Option<Color>),
    Cursor((usize, (usize, usize))),
    Empty,
}

use crossterm::{
    cursor::{MoveTo, MoveToColumn},
    style::{Attribute, SetAttributes},
    style::{Print, ResetColor, SetForegroundColor},
};

impl Command for Lense<'_> {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        use Lense::*;

        match self {
            NewLine => MoveToColumn(0).write_ansi(out),
            Encoded(display) => Print(display).write_ansi(out),
            Color(Some(color)) => SetForegroundColor(*color).write_ansi(out),
            Color(None) => ResetColor.write_ansi(out),
            Cursor((_, (x, y))) => MoveTo(*x as u16, *y as u16).write_ansi(out),
            Active => SetAttributes(From::from(
                [Attribute::Underlined, Attribute::Bold].as_ref(),
            ))
            .write_ansi(out),
            Inactive => SetAttributes(From::from(Attribute::Reset)).write_ansi(out),
            Group(slice) => slice
                .iter()
                .map(|block| block.write_ansi(out))
                .find(|result| result.is_err())
                .unwrap_or(Ok(())),
            Empty => Ok(()),
        }
    }
}

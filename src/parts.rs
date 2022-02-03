pub struct Located<T>((usize, usize), T);

use std::fmt;

use crossterm::Command;

use crossterm::{cursor::MoveTo, style::Print};

use num_traits::FromPrimitive;

impl<T: fmt::Display> Command for Located<T> {
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        let x = u16::from_usize(self.0 .0).ok_or(fmt::Error)?;
        let y = u16::from_usize(self.0 .1).ok_or(fmt::Error)?;

        MoveTo(x, y)
            .write_ansi(out)
            .and(Print(&self.1).write_ansi(out))
    }
}

pub struct Cells<I> {
    inner: I,
}

impl<'bytes, 'iter, I, T> IntoIterator for &'iter Cells<I>
where
    'bytes: 'iter,
    &'iter I: Iterator<Item = &'iter T> + 'iter,
    T: 'iter,
{
    type IntoIter = &'iter I;
    type Item = &'iter T;

    fn into_iter(self: &'iter Cells<I>) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'bytes, 'iter, I, T> Command for &'iter Cells<I>
where
    'bytes: 'iter,
    &'iter I: Iterator<Item = &'iter T> + 'iter,
    T: Command + 'iter,
{
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        self.into_iter()
            .map(|cell| cell.write_ansi(out))
            .fold(Ok(()), |_, next| next)
    }
}

use std::io::{stdout, Write};

use crossterm::{
    cursor::{Hide, Show},
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
}

#[derive(PartialEq)]
pub enum Outcome {
    Continue,
    Done,
    Exit(Option<i32>),
}

pub trait Screen: Sized {
    type View: Command;

    fn display(&self) -> Self::View;

    fn cells<'bytes, 'iter>(&'iter self) -> &'iter Cells<&'iter Self>
    where
        'bytes: 'iter,
        &'iter Self: Iterator<Item = &'iter Self::View>,
        &'iter Self::View: Command,
    {
        &Cells { inner: self }
    }

    fn hello<'bytes, 'iter, I>(self: &'iter Self) -> Result<Outcome, Error>
    where
        'bytes: 'iter,
        &'iter Self: Iterator<Item = &'iter Self::View>,
        &'iter Self::View: Command,
    {
        let mut output = stdout();

        let cells = self.cells();

        execute!(output, EnterAlternateScreen, Hide, self.cells())?;

        Ok(Outcome::Done)
    }

    /*
    fn watch<'bytes, 'iter>(&'iter self) -> Result<Outcome, Error>
    where
        'bytes: 'iter,
        &'iter Self: Iterator<Item = &'iter Self::View>,
        &'iter Self::View: Command,
    {
        let mut outcome = Outcome::Continue;

        let mut output = stdout();

        let cells = self.cells();

        execute!(output, EnterAlternateScreen, Hide, &cells)?;

        enable_raw_mode()?;

        loop {
            let event = read()?;

            match self.handle_event(&event) {
                next @ Outcome::Done | next @ Outcome::Exit(_) => {
                    outcome = next;
                    break;
                }
                _ => {}
            };

            let cells = self.cells();

            execute!(output, Clear(ClearType::All), MoveTo(0, 0), &cells)?;

            output.flush()?;
        }

        disable_raw_mode()?;

        execute!(output, Show, LeaveAlternateScreen)?;

        Ok(outcome)
    }
    */

    fn handle(&mut self, event: &Event) -> Outcome {
        self.handle_common(event)
    }

    fn handle_common(&mut self, event: &Event) -> Outcome {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => Outcome::Exit(None),
            _ => Outcome::Continue,
        }
    }

    fn handle_inner(&mut self, _: &Event) -> Outcome {
        Outcome::Continue
    }

    fn handle_event(&mut self, event: &Event) -> Outcome {
        match self.handle_inner(event) {
            Outcome::Continue => self.handle(event),
            outcome @ _ => outcome,
        }
    }
}

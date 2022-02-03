pub struct Locations<T>(T);

impl<T> Iterator for Locations<T>
where
    T: IntoIterator,
    T::Item: Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.into_iter().next()
    }
}

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

pub trait View: Command {}

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

pub trait Context: View {
    fn watch(&self) -> Result<Outcome, std::io::Error> {
        let mut outcome = Outcome::Continue;

        let mut output = stdout();

        execute!(output, EnterAlternateScreen, Hide, self)?;

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

            execute!(output, Clear(ClearType::All), MoveTo(0, 0), self)?;

            output.flush()?;
        }

        disable_raw_mode()?;

        execute!(output, Show, LeaveAlternateScreen)?;

        Ok(outcome)
    }

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

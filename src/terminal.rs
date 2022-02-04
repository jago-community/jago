#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
}

use ::{
    crossterm::{
        cursor::{Hide, Show},
        event::read,
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    std::io::{stdout, Write},
};

use crate::document::{Document, Operation, Span};

use ::{
    crossterm::{style::Print, Command},
    std::fmt::Display,
};

pub fn watch<'a, U: Display + Span>(mut document: Document<U>) -> Result<Operation, Error> {
    let mut output = stdout();

    execute!(output, EnterAlternateScreen, Hide, &document)?;

    enable_raw_mode()?;

    let mut outcome = Operation::Continue;

    loop {
        let event = read()?;

        match document.handle(&event) {
            next @ Operation::Done | next @ Operation::Exit(_) => {
                outcome = next;
                break;
            }
            _ => {}
        };

        execute!(output, EnterAlternateScreen, Hide, &document)?;
    }

    disable_raw_mode()?;

    execute!(output, Show, LeaveAlternateScreen)?;

    output.flush()?;

    Ok(outcome)
}

/*
use std::borrow::Cow;

pub struct Directive<'a, A: ToOwned + ?Sized + 'a>(Cow<'a, A>);

impl<'a, A: Clone> From<&'a A> for Directive<'a, A> {
    fn from(a: &'a A) -> Self {
        Directive(Cow::Borrowed(a))
    }
}

impl<'a> From<&'a str> for Directive<'a, str> {
    fn from(a: &'a str) -> Self {
        Directive(Cow::Borrowed(a))
    }
}


pub trait AsCommand<'a> {
    type Base: Command;

    fn as_command(&self) -> Self::Base;

    fn as_print(&'a self) -> Print<&'a Self>
    where
        Self: Display,
    {
        Print(self)
    }
}

impl<'a, T: AsCommand<'a>> AsCommand<'a> for &'a T {
    type Base = T::Base;

    fn as_command(&self) -> Self::Base {
        (**self).as_command()
    }
}

impl<'a, X: Clone + AsCommand<'a>> AsCommand<'a> for Directive<'a, X> {
    type Base = X::Base;

    fn as_command(&self) -> Self::Base {
        self.0.as_command()
    }
}

impl<'a> AsCommand<'a> for Directive<'a, str> {
    type Base = Print<Cow<'a, str>>;

    fn as_command(&self) -> Self::Base {
        Print(self.0.clone())
    }
}

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

pub trait Handle {
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

impl<S> Handle for S {}
*/

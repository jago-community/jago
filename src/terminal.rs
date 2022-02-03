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

use ::{
    crossterm::{style::Print, Command},
    std::fmt::Display,
};

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

/*
pub struct And<'a, X, Y: Iterator<Item = &'a X>>(Y, std::marker::PhantomData<&'a X>);

impl<'a, A: Clone> Directive<'a, A> {
    fn and<B>(b: B) -> And<'a, A, B>
    where
        B: Iterator<Item = &'a A>,
    {
        And::from(b)
    }
}

impl<'a, A, B> From<B> for And<'a, A, B>
where
    B: Iterator<Item = &'a A>,
{
    fn from(b: B) -> Self {
        Self(b, Default::default())
    }
}

impl<'a, X, Y> AsCommand<'a> for And<'a, X, Y>
where
    X: AsCommand<'a>,
    Y: Iterator<Item = &'a X>,
{
    type Base = X::Base;

    fn as_command(&'a self) -> Self::Base {
        let collection = self.0.collect::<Vec<_>>();

        Self::Base::from(collection.iter())
    }
}
*/

#[derive(PartialEq)]
pub enum Outcome {
    Continue,
    Done,
    Exit(Option<i32>),
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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
}

use ::{
    crossterm::{
        cursor::{Hide, Show},
        event::read,
        queue,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    std::io::{stdout, Write},
};

pub fn watch<'a>(buffer: impl Iterator<Item = Directive<'a, str>>) -> Result<Outcome, Error> {
    let mut directives = buffer.collect::<Vec<_>>();

    let mut output = stdout();

    queue!(output, EnterAlternateScreen, Hide)?;

    directives
        .iter()
        .find_map(|directive| queue!(output, directive.as_command()).err());

    output.flush()?;

    enable_raw_mode()?;

    let mut outcome = Outcome::Continue;

    loop {
        let event = read()?;

        match directives.handle_event(&event) {
            next @ Outcome::Done | next @ Outcome::Exit(_) => {
                outcome = next;
                break;
            }
            _ => {}
        };

        queue!(output, EnterAlternateScreen, Hide)?;

        directives
            .iter()
            .find_map(|directive| queue!(output, directive.as_command()).err());

        output.flush()?;
    }

    disable_raw_mode()?;

    queue!(output, Show, LeaveAlternateScreen)?;

    output.flush()?;

    Ok(outcome)
}

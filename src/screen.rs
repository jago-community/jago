use std::fmt::{self, Display};

#[derive(PartialEq)]
pub enum Outcome {
    Continue,
    Done,
    Exit(Option<i32>),
}

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::Print,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

use std::io::{stdout, Write};

pub trait Screen {
    type Filter: Display;

    fn screen(&self) -> &Self::Filter;

    fn view(&self, out: &mut impl fmt::Write) -> fmt::Result {
        write!(out, "{}", self.screen())
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

    fn watch(&mut self) -> Result<Outcome, std::io::Error> {
        let mut outcome = Outcome::Continue;

        let mut output = stdout();

        execute!(output, EnterAlternateScreen, Hide, Print(self.screen()))?;

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

            execute!(
                output,
                Clear(ClearType::All),
                MoveTo(0, 0),
                Print(self.screen())
            )?;

            output.flush()?;
        }

        disable_raw_mode()?;

        execute!(output, Show, LeaveAlternateScreen)?;

        Ok(outcome)
    }
}

impl<D: Display> Screen for D {
    type Filter = D;

    fn screen(&self) -> &Self::Filter {
        self
    }
}

use std::path;

pub struct Resource<'a> {
    path: path::PathBuf,
    lifetime: std::marker::PhantomData<&'a ()>,
}

impl<'a> Screen for Resource<'a> {
    type Filter = path::Display<'a>;

    fn screen(&self) -> &Self::Filter {
        let display = self.path.display();

        display.as_ref()
    }
}

#[derive(Default)]
pub struct Context(usize);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
}

use crate::traits::{Lense, Viewer};

impl<'a> Viewer for &'a Context {
    fn view(&self) -> Lense {
        Lense::Encoded(Box::new("Hello, stranger."))
    }
}

use crate::traits::{Handler, Outcome};

use crossterm::event::{Event, KeyCode, KeyEvent};

impl<'a> Handler for &'a Context {
    fn handle(&mut self, event: &Event) -> Outcome {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => Outcome::Done,
            _ => self.handle_common(event),
        }
    }
}

use std::io::{stdout, Write};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::read,
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

impl Context {
    pub fn watch(&self, mut item: impl Viewer + Handler) -> Result<Outcome, Error> {
        let mut outcome = Outcome::Continue;

        let mut output = stdout();

        execute!(output, EnterAlternateScreen, Hide, item.view())?;

        enable_raw_mode()?;

        loop {
            let event = read()?;

            match item.handle(&event) {
                next @ Outcome::Done | next @ Outcome::Exit(_) => {
                    outcome = next;
                    break;
                }
                _ => {}
            };

            execute!(output, Clear(ClearType::All), MoveTo(0, 0), item.view())?;

            output.flush()?;
        }

        disable_raw_mode()?;

        execute!(output, Show, LeaveAlternateScreen)?;

        Ok(outcome)
    }
}

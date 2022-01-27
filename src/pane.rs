pub struct Pane<Inner> {
    inner: Inner,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
}

impl<Inner: std::fmt::Display> From<Inner> for Pane<Inner> {
    fn from(inner: Inner) -> Self {
        Self { inner }
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
    Command,
};

impl<Inner> Pane<Inner> {
    pub fn view(inner: impl Into<Self>) -> Result<(), Error> {
        let mut this = inner.into();

        let mut output = stdout();

        execute!(output, EnterAlternateScreen, Hide, &this)?;

        enable_raw_mode()?;

        loop {
            let event = read()?;

            if !this.handle(&event) {
                break;
            }

            execute!(output, Clear(ClearType::All), MoveTo(0, 0), &this)?;

            output.flush()?;
        }

        disable_raw_mode()?;

        execute!(output, Show, LeaveAlternateScreen)?;

        Ok(())
    }
}

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

impl<Inner> Pane<Inner> {
    fn handle(&mut self, event: &Event) -> bool {
        let mut fine = true;

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => {
                fine = false;
            }
            _ => {}
        };

        fine
    }
}

use std::fmt;

use crossterm::style::Print;

impl<Inner> Command for Pane<Inner> {
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        Print("Hello, stranger.").write_ansi(out)
    }
}

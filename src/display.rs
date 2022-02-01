use crate::{handle::Outcome, traits::Screen};

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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
}

pub fn watch<'a>(mut item: impl Screen<'a>) -> Result<Outcome, Error> {
    let mut outcome = Outcome::Continue;

    let mut output = stdout();

    execute!(output, EnterAlternateScreen, Hide, item.cells())?;

    enable_raw_mode()?;

    loop {
        let event = read()?;

        match item.handle_event(&event) {
            next @ Outcome::Done | next @ Outcome::Exit(_) => {
                outcome = next;
                break;
            }
            _ => {}
        };

        execute!(output, Clear(ClearType::All), MoveTo(0, 0), item.cells())?;

        output.flush()?;
    }

    disable_raw_mode()?;

    execute!(output, Show, LeaveAlternateScreen)?;

    Ok(outcome)
}

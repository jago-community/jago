#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
}

use ::{
    crossterm::{
        cursor::{Hide, MoveTo, Show},
        event::read,
        execute,
        terminal::{
            disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
            LeaveAlternateScreen,
        },
    },
    std::io::stdout,
};

use crate::buffer::{Buffer, Outcome};

pub fn watch<I>(buffer: &Buffer<I>) -> Result<(), Error> {
    let mut outcome = Outcome::Continue;

    let mut output = stdout();

    execute!(output, EnterAlternateScreen, Hide, buffer.directives())?;

    enable_raw_mode()?;

    loop {
        let event = read()?;

        match buffer.handle_event(&event) {
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
            buffer.directives()
        )?;

        output.flush()?;
    }

    disable_raw_mode()?;

    execute!(output, Show, LeaveAlternateScreen)?;

    Ok(outcome)
}

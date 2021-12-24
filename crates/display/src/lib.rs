mod document;

use context::Context;

use std::{io::stdout, iter::Peekable};

use crossterm::{
    cursor::{MoveDown, MoveLeft, MoveRight, MoveTo, MoveUp, RestorePosition, SavePosition},
    event::{read, Event, KeyCode, KeyEvent},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::document::Document;

pub fn handle(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &Context,
) -> Result<(), Error> {
    match input.peek() {
        Some(name) if name == "display" => {
            drop(input.next());
        }
        _ => {}
    };

    let mut output = stdout();

    execute!(output, EnterAlternateScreen)?;

    enable_raw_mode()?;

    loop {
        let mut target = context.target();

        context.read(&mut target)?;

        disable_raw_mode()?;

        execute!(
            output,
            SavePosition,
            MoveTo(0, 0),
            Print(String::from_utf8_lossy(&target)),
            Document::from(target.as_ref()),
            RestorePosition,
        )?;

        enable_raw_mode()?;

        let event = read()?;

        // context.write(format!("{:?}\n", event))?;

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => {
                break;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('h'),
                ..
            }) => {
                execute!(output, MoveLeft(1))?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                ..
            }) => {
                execute!(output, MoveRight(1))?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                ..
            }) => {
                execute!(output, MoveDown(1))?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                ..
            }) => {
                execute!(output, MoveUp(1))?;
            }
            _ => {}
        };
    }

    disable_raw_mode()?;

    execute!(output, LeaveAlternateScreen)?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("Context {0}")]
    Context(#[from] context::Error),
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

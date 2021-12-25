mod document;

use context::Context;

use std::{
    env::current_dir,
    fs::File,
    io::{stdout, Read},
    iter::Peekable,
};

use crossterm::{
    cursor::{
        CursorShape, MoveDown, MoveLeft, MoveRight, MoveTo, MoveUp, RestorePosition, SavePosition,
        SetCursorShape,
    },
    event::{read, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::document::Document;

pub fn handle(
    input: &mut Peekable<impl Iterator<Item = String>>,
    _: &Context,
) -> Result<(), Error> {
    match input.peek() {
        Some(name) if name == "display" => {
            drop(input.next());
        }
        _ => {}
    };

    let mut source = vec![];

    let target = current_dir().map_err(Error::from).and_then(|directory| {
        directory
            .file_stem()
            .ok_or_else(|| Error::Incomplete)
            .map(|file_stem| directory.join(file_stem))
    })?;

    let mut file = File::open(target)?;

    file.read_to_end(&mut source)?;

    let mut output = stdout();

    execute!(
        output,
        EnterAlternateScreen,
        SetCursorShape(CursorShape::Line)
    )?;

    enable_raw_mode()?;

    loop {
        disable_raw_mode()?;

        let position = crossterm::cursor::position()?;

        let document = Document::new(&source, position);

        execute!(
            output,
            SavePosition,
            MoveTo(0, 0),
            document,
            RestorePosition,
        )?;

        enable_raw_mode()?;

        let event = read()?;

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

    execute!(
        output,
        SetCursorShape(CursorShape::Block),
        LeaveAlternateScreen
    )?;

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

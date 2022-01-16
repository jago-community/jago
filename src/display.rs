use std::io::{stdout, Write};

use crossterm::{
    cursor::{CursorShape, SetCursorShape},
    event::{read, Event, KeyCode, KeyEvent},
    execute, queue,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::slice::Slice;

pub fn buffer(source: &[u8]) -> Result<(), Error> {
    let mut slice = Slice::from(source);

    let mut output = stdout();

    execute!(
        output,
        EnterAlternateScreen,
        SetCursorShape(CursorShape::UnderScore),
        &slice,
    )?;

    enable_raw_mode()?;

    loop {
        disable_raw_mode()?;

        execute!(output, &slice)?;

        enable_raw_mode()?;

        let event = read()?;

        slice.handle(&event);

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => {
                break;
            }
            _ => {}
        };

        queue!(output, &slice)?;

        output.flush()?;
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
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

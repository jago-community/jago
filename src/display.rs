use std::io::{stdout, Write};

use crossterm::{
    cursor::{CursorShape, SetCursorShape},
    event::{read, Event, KeyCode, KeyEvent},
    execute, queue,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::buffer::Buffer;

pub fn buffer(source: &[u8]) -> Result<(), Error> {
    let mut buffer = Buffer::new(source);

    let mut output = stdout();

    execute!(
        output,
        EnterAlternateScreen,
        SetCursorShape(CursorShape::UnderScore),
        &buffer,
    )?;

    enable_raw_mode()?;

    loop {
        disable_raw_mode()?;

        execute!(output, &buffer)?;

        enable_raw_mode()?;

        let event = read()?;

        buffer.handle(&event);

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => {
                break;
            }
            _ => {}
        };

        queue!(output, &buffer)?;

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

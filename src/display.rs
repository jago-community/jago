use std::io::{stdout, Write};

use crossterm::{
    cursor::{CursorShape, SetCursorShape},
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::{buffer::Buffer, slice::Slice};

pub fn buffer() -> Result<(), Error> {
    let sources = &[
        include_bytes!("../poems/chris-abani/the-new-religion").as_ref(),
        include_bytes!("../poems/eltheridge-knight/haiku/1").as_ref(),
    ];

    let mut index = 0;

    let mut buffer = Buffer::from(sources[index]);

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

        let current = buffer.read_bytes();

        let slice = Slice::from(current.as_ref());

        execute!(output, &buffer)?;

        enable_raw_mode()?;

        let event = read()?;

        if buffer.handle(slice, &event) {
            break;
        }

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('n'),
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => {
                index = (index + 1) % 2;
                buffer = Buffer::from(sources[index]);
            }
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

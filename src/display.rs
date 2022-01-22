use std::{
    io::{stdout, Write},
    path::{Path, PathBuf},
};

use crossterm::{
    cursor::{CursorShape, MoveTo, SetCursorShape},
    event::read,
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

use crate::directory::Directory;

pub fn directory(path: &Path) -> Result<Option<PathBuf>, Error> {
    let mut directory = Directory::from(path);

    let mut result = None;

    let mut output = stdout();

    let block = directory.write_terminal()?;

    execute!(output, EnterAlternateScreen, block)?;

    enable_raw_mode()?;

    loop {
        let event = read()?;

        match directory.handle(&event) {
            Err(_) => {
                break;
            }
            Ok(Some(selected)) => {
                result = Some(selected.to_path_buf());
                break;
            }
            _ => {}
        }

        let block = directory.write_terminal()?;

        execute!(output, Clear(ClearType::All), MoveTo(0, 0), block)?;

        output.flush()?;
    }

    disable_raw_mode()?;

    execute!(output, LeaveAlternateScreen)?;

    Ok(result)
}

use std::{fs::OpenOptions, io::Read};

use crate::buffer::Buffer;

pub fn file(path: &Path) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(path)?;

    let mut bytes = vec![];

    file.read_to_end(&mut bytes)?;

    let mut buffer = Buffer::from(bytes.as_ref());

    let mut output = stdout();

    let block = buffer.write_terminal()?;

    execute!(output, EnterAlternateScreen, block,)?;

    enable_raw_mode()?;

    loop {
        let event = read()?;

        if let Err(_) = buffer.handle(&event) {
            break;
        }

        let block = buffer.write_terminal()?;

        execute!(output, Clear(ClearType::All), MoveTo(0, 0), block)?;

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
    #[error("Directory {0}")]
    Directory(#[from] crate::directory::Error),
    #[error("Directory {0}")]
    Buffer(#[from] crate::buffer::Error),
}

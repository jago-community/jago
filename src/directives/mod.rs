mod color_picker;
mod shell;
mod traits;

pub use shell::Shell;
pub use traits::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

pub use traits::{Directive, Op};

use ::{
    crossterm::{
        cursor::{Hide, Show},
        event::read,
        execute, queue,
        terminal::{
            disable_raw_mode, enable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen,
        },
    },
    std::io::{stdout, Write},
};

pub fn watch<'a, D: Directive>(mut directive: D, screen: bool) -> Result<Op, Error> {
    let (x, y) = size()?;

    match directive.handle_event(&Event::Resize(x, y)) {
        op @ Op::Done | op @ Op::Exit(_, _) => {
            return Ok(op);
        }
        _ => {}
    };

    let mut output = stdout();

    if screen {
        queue!(output, EnterAlternateScreen)?;
    }

    execute!(output, Hide, &directive)?;

    enable_raw_mode()?;

    let mut op = Op::Continue;

    loop {
        let event = read()?;

        match directive.handle_event(&event) {
            next @ Op::Done | next @ Op::Exit(_, _) => {
                op = next;
                break;
            }
            _ => {}
        };

        execute!(output, &directive)?;
    }

    disable_raw_mode()?;

    if screen {
        queue!(output, LeaveAlternateScreen)?;
    }

    execute!(output, Show)?;

    output.flush()?;

    Ok(op)
}

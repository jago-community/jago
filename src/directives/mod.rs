pub mod buffer;
pub mod shell;
mod traits;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

pub use traits::{Directive, Op};

use ::{
    crossterm::{
        cursor::{Hide, Show},
        event::{read, Event},
        execute,
        terminal::{
            disable_raw_mode, enable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen,
        },
    },
    std::io::{stdout, Write},
};

pub fn watch<'a, D: Directive>(mut directive: D) -> Result<Op, Error> {
    let (x, y) = size()?;

    match directive.handle_event(&Event::Resize(x, y)) {
        op @ Op::Done | op @ Op::Exit(_) => {
            return Ok(op);
        }
        _ => {}
    };

    let mut output = stdout();

    execute!(output, EnterAlternateScreen, Hide, &directive)?;

    enable_raw_mode()?;

    let mut op = Op::Continue;

    loop {
        let event = read()?;

        match directive.handle_event(&event) {
            next @ Op::Done | next @ Op::Exit(_) => {
                op = next;
                break;
            }
            _ => {}
        };

        execute!(output, EnterAlternateScreen, Hide, &directive)?;
    }

    disable_raw_mode()?;

    execute!(output, Show, LeaveAlternateScreen)?;

    output.flush()?;

    Ok(op)
}

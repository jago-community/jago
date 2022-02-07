#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

use ::{
    crossterm::{
        cursor::{Hide, Show},
        event::{Event, EventStream},
        execute, queue,
        terminal::{
            disable_raw_mode, enable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen,
        },
    },
    futures::stream::StreamExt,
    std::io::{stdout, Write},
    tokio::runtime,
};

use crate::directives::{Directive, Op};

pub fn watch<D: Directive>(mut directive: D) -> Result<Op, Error> {
    let (x, y) = size()?;

    match directive.handle_event(&Event::Resize(x, y)) {
        op @ Op::Done | op @ Op::Exit(_, _) => {
            return Ok(op);
        }
        _ => {}
    };

    let mut output = stdout();

    execute!(output, EnterAlternateScreen, Hide, &directive)?;

    enable_raw_mode()?;

    let mut op = Op::Continue;

    let runtime = runtime::Builder::new_current_thread().build()?;

    let mut reader = EventStream::new();

    runtime.block_on(async {
        reader
            .map(|event| directive.handle_event(event))
            .inspect(|_| {
                queue!(output, &directive)?;
            })
            .take_until(|op| op.stop())
            .for_each_concurrent(None, |op| async move {
                match directive.handle_event(&event) {
                    next @ Op::Done | next @ Op::Exit(_, _) => {
                        op = next;
                    }
                    _ => {}
                };

                output.flush();
            })
            .await;
    });

    disable_raw_mode()?;

    execute!(output, LeaveAlternateScreen, Show)?;

    output.flush()?;

    Ok(op)
}

pub fn hhhhhh<'a, D: Directive>(mut directive: D, screen: bool) -> Result<Op, Error> {
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

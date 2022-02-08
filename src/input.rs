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
    futures::{
        future,
        stream::{self, StreamExt},
    },
    std::io::{stdout, Write},
    tokio::runtime,
};

use crate::directives::{Directive, Op};

pub fn watch<D: Directive + Clone>(mut directive: D) -> Result<Op, Error> {
    let runtime = runtime::Builder::new_current_thread().build()?;

    runtime.block_on(async move {
        let reader = EventStream::new();

        let (x, y) = size()?;

        match directive.handle_event(&Event::Resize(x, y)) {
            op @ Op::Done | op @ Op::Exit(_, _) => {
                return Ok(op);
            }
            _ => {}
        };

        let mut out = stdout();

        execute!(&out, EnterAlternateScreen, Hide, &directive)?;

        enable_raw_mode()?;

        let mut handle = directive.cloned();
        let mut context = directive.cloned();

        let view = directive.cloned();

        reader
            .flat_map(|item| stream::iter(item.ok()))
            .map(|event| handle.handle_event(&event))
            .take_while(|op| future::ready(!op.stop()))
            .for_each(move |_| {
                context.before();

                queue!(out, &view).expect("dop");

                out.flush().expect("gah");

                future::ready(())
            })
            .await;

        disable_raw_mode()?;

        let mut out = stdout();

        execute!(&out, LeaveAlternateScreen, Show)?;

        out.flush()?;

        Ok(Op::Continue)
    })
}

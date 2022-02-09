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

use crate::view::Buffer;

pub fn watch(buffer: Buffer) -> Result<(), Error> {
    let runtime = runtime::Builder::new_current_thread().build()?;

    runtime.block_on(async move {
        let reader = EventStream::new();

        let (x, y) = size()?;

        let mut out = stdout();

        execute!(&out, EnterAlternateScreen, Hide, &buffer)?;

        enable_raw_mode()?;

        reader
            .flat_map(|item| stream::iter(item.ok()))
            .for_each(move |item| {
                queue!(&mut out, &buffer).expect("hello");

                out.flush().expect("gah");

                future::ready(())
            })
            .await;

        disable_raw_mode()?;

        let mut out = stdout();

        execute!(&out, LeaveAlternateScreen, Show)?;

        out.flush()?;

        Ok(())
    })
}

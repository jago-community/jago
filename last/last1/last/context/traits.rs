#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

use ::{
    crossterm::{
        cursor::{Hide, Show},
        event::EventStream,
        style::Print,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand, QueueableCommand,
    },
    futures::{
        future,
        stream::{self, StreamExt},
    },
    std::io::{self, Write},
    tokio::runtime::Builder as Runtime,
};

pub trait Screen: Write {
    fn watch(&self) -> io::Result<()> {
        let runtime = Runtime::new_current_thread().build()?;

        runtime.block_on(async move {
            let reader = EventStream::new();

            self.execute(EnterAlternateScreen)?.execute(Hide)?;

            enable_raw_mode()?;

            reader
                .flat_map(|item| stream::iter(item.ok()))
                .map(|event| self.queue(Print(format!("{:?}\n", event))))
                .map(|res| res.and(self.flush()))
                .for_each(move |_| future::ready(()))
                .await;

            self.flush()?;

            disable_raw_mode()?;

            self.execute(LeaveAlternateScreen)?.execute(Show)?;

            Ok(())
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

use ::{
    crossterm::{
        cursor::{Hide, Show},
        event::EventStream,
        terminal::{
            disable_raw_mode, enable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen,
        },
        ExecutableCommand, QueueableCommand,
    },
    futures::{
        future,
        stream::{self, StreamExt},
    },
    std::io::Write,
    tokio::runtime::Builder as Runtime,
};

pub trait Screen: Write {
    fn watch(&mut self) -> Result<(), Error> {
        let runtime = Runtime::new_current_thread().build()?;

        runtime.block_on(async move {
            let reader = EventStream::new();

            let (x, y) = size()?;

            //match directive.handle_event(&Event::Resize(x, y)) {
            //handle @ _ if handle.stop() => return Ok(handle),
            //_ => {}
            //};

            self.queue(EnterAlternateScreen)?.execute(Hide)?;

            enable_raw_mode()?;
            /*
                        reader
                            .flat_map(|item| stream::iter(item.ok()))
                            .map(|event| handle.handle_event(&event))
                            .take_while(|op| future::ready(!op.stop()))
                            .map(|a| (context.before(), a))
                            .for_each(move |(before_result, _)| {
                                before_result.expect("good bye");

                                // queue!(&mut out, &view).expect("hello");

                                self.flush().expect("gah");

                                future::ready(())
                            })
                            .await;
            */
            disable_raw_mode()?;

            self.queue(LeaveAlternateScreen)?.execute(Show)?;

            Ok(())
        })
    }
}

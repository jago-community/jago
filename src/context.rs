#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialize {0}")]
    Serialize(#[from] crate::serialize::Error),
    #[error("View {0}")]
    View(#[from] crate::view::Error),
}

use crate::serialize::Serializer;

use ::{
    crossterm::{
        cursor::{Hide, MoveTo, Show},
        event::{Event, EventStream},
        terminal::{
            disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
            LeaveAlternateScreen,
        },
    },
    futures::{
        future,
        stream::{self, StreamExt},
    },
    tokio::runtime,
};

use crate::{Directive, Directives, Handle, View};

pub trait Context: View + Handle {
    fn watch(&mut self) -> Result<(), Error> {
        let mut buffer = std::io::stdout();

        let mut serializer = Serializer::new(&mut buffer);

        let runtime = runtime::Builder::new_current_thread().build()?;

        runtime.block_on(async move {
            let reader = EventStream::new();

            let (x, y) = size()?;

            if self.handle(&Event::Resize(x, y)).stop() {
                return Ok(());
            }

            //serializer.consume(EnterAlternateScreen)?;
            //serializer.consume(Hide)?;

            self.serialize(&mut serializer)?;

            serializer.flush()?;

            enable_raw_mode()?;

            let (_, _) = reader
                .flat_map(|result| stream::iter(result.ok()))
                .map(|event| self.handle(&event))
                .map(|directives| -> Result<Directives, Error> {
                    serializer.consume(Clear(ClearType::All))?;
                    serializer.consume(MoveTo(0, 0))?;

                    self.serialize(&mut serializer)?;

                    serializer.flush()?;

                    Ok(directives)
                })
                .flat_map(|result| stream::iter(result))
                .filter(|directives| future::ready(directives.stop()))
                .into_future()
                .await;

            disable_raw_mode()?;

            //serializer.consume(Show)?;
            //serializer.consume(LeaveAlternateScreen)?;

            Ok(())
        })
    }
}

impl<'a, C> Context for C where C: Handle + View {}

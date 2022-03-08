#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

use ::{
    crossterm::{
        cursor::{Hide, MoveTo, Show},
        event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
        terminal::{
            disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
            LeaveAlternateScreen,
        },
        QueueableCommand,
    },
    std::io::{stdout, Write},
    tokio::runtime,
};

use crate::{Context, Handle};

pub fn watch(context: &Context) -> Result<(), Error> {
    let runtime = runtime::Builder::new_current_thread().build()?;

    runtime.block_on(async {
        let mut buffer = stdout();

        enable_raw_mode()?;

        buffer
            .queue(EnterAlternateScreen)?
            .queue(Hide)?
            .queue(MoveTo(0, 0))?
            .queue(context)?
            .flush()?;

        loop {
            match event::read() {
                Ok(event) => match event {
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                    }) => break,
                    _ => context.handle(&event),
                },
                _ => break,
            };

            buffer
                .queue(Clear(ClearType::All))?
                .queue(MoveTo(0, 0))?
                .queue(context)?
                .flush()?;
        }

        buffer.queue(Show)?.queue(LeaveAlternateScreen)?.flush()?;

        disable_raw_mode()?;

        Ok(())
    })
}

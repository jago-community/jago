#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

pub trait Directive {
    fn stop(&self) -> bool;
}

use bitflags::bitflags;

bitflags! {
    pub struct Directives: u32 {
        const STOP = 0b00000001;
    }
}

impl Directive for Directives {
    fn stop(&self) -> bool {
        self.contains(Directives::STOP)
    }
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

use crate::Document;

pub fn watch(document: Document) -> Result<(), Error> {
    let runtime = runtime::Builder::new_current_thread().build()?;

    runtime.block_on(async {
        let mut buffer = stdout();

        enable_raw_mode()?;

        buffer
            .queue(EnterAlternateScreen)?
            .queue(Hide)?
            .queue(MoveTo(0, 0))?;

        document.queue_ansi(&mut buffer)?;

        buffer.flush()?;

        loop {
            let directives = match event::read() {
                Ok(event) => match event {
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                    }) => Directives::STOP,
                    _ => Directives::empty(),
                },
                _ => Directives::STOP,
            };

            if directives.stop() {
                break;
            } else {
                buffer.queue(Clear(ClearType::All))?.queue(MoveTo(0, 0))?;

                document.queue_ansi(&mut buffer)?;

                buffer.flush()?;
            }
        }

        buffer.queue(Show)?.queue(LeaveAlternateScreen)?.flush()?;

        disable_raw_mode()?;

        Ok(())
    })
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("NoHome")]
    NoHome,
    #[error("SetLogger {0}")]
    SetLogger(#[from] log::SetLoggerError),
    #[error("SetLogger {0}")]
    Environment(#[from] environment::Error),
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

use ::{
    simplelog::{
        ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
    },
    std::fs::OpenOptions,
};

pub fn before() -> Result<(), Error> {
    let path = environment::logs_directory(true).map(|path| path.join("current"))?;

    let file = OpenOptions::new().create(true).append(true).open(path)?;

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Warn,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(LevelFilter::Info, Config::default(), file),
    ])
    .map_err(Error::from)
}

use ::{
    crossterm::{
        cursor::{MoveTo, MoveToNextLine},
        event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
        style::Print,
        terminal::{
            disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
            LeaveAlternateScreen,
        },
        Command, QueueableCommand,
    },
    std::{
        fmt,
        io::{stdout, Write},
        sync::{Arc, Mutex},
    },
    tokio::runtime,
};

use context::{Directive, Directives, Handle};

pub struct Context {
    inner: Arc<Mutex<context::Context>>,
}

impl From<context::Context> for Context {
    fn from(inner: context::Context) -> Self {
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }
}

impl Command for Context {
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        let inner = self.inner.lock().map_err(|_| fmt::Error)?;

        for c in inner.buffer().iter() {
            if c == &'\n' {
                MoveToNextLine(1).write_ansi(out)?;
            } else {
                Print(*c).write_ansi(out)?;
            }
        }

        Ok(())
    }
}

impl Handle for Context {
    type Event = Event;
    type Directive = Directives;

    fn handle(&self, event: &Self::Event) -> Self::Directive {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
            }) => Directives::STOP,
            _ => Directives::empty(),
        }
    }
}

pub fn watch(context: impl Into<Context>) -> Result<(), Error> {
    let context = context.into();

    let runtime = runtime::Builder::new_current_thread().build()?;

    runtime.block_on(async {
        let mut buffer = stdout();

        enable_raw_mode()?;

        buffer
            .queue(EnterAlternateScreen)?
            .queue(MoveTo(0, 0))?
            .queue(&context)?
            .flush()?;

        loop {
            let directives = match event::read() {
                Ok(event) => context.handle(&event),
                _ => break,
            };

            buffer
                .queue(Clear(ClearType::All))?
                .queue(MoveTo(0, 0))?
                .queue(&context)?
                .flush()?;

            if directives.stop() {
                break;
            }
        }

        buffer.queue(LeaveAlternateScreen)?.flush()?;

        disable_raw_mode()?;

        Ok(())
    })
}

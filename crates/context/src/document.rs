use crate::pipe;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Poisoned")]
    Poisoned,
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
    #[error("Pipe {0}")]
    Pipe(#[from] pipe::Error),
}

use std::{
    io::{stderr, Stderr},
    sync::{Arc, Mutex},
};

use pipe::Pipe;

#[derive(Debug)]
pub struct Document {
    buffer: Arc<Mutex<Vec<u8>>>,
    output: Stderr,
    pipe: Pipe<Stderr>,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(vec![])),
            output: stderr(),
            pipe: stderr().into(),
        }
    }
}

impl Document {
    fn record(&self, expression: Expression) -> Result<(), Error> {
        use crossterm::execute;

        //execute!(self.output.lock(), expression).map_err(Error::from)

        let mut pipe = self.pipe.inner()?;

        execute!(pipe, expression).map_err(Error::from)
    }
}

enum Expression<'a> {
    Log(&'a log::Record<'a>),
}

impl<'a> crossterm::Command for Expression<'a> {
    fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            Expression::Log(record) => {
                write!(
                    f,
                    "{}{}{}{}\n",
                    crossterm::style::SetForegroundColor(match record.level() {
                        log::Level::Error => crossterm::style::Color::Red,
                        log::Level::Warn => crossterm::style::Color::Yellow,
                        log::Level::Info => crossterm::style::Color::Green,
                        log::Level::Debug => crossterm::style::Color::Blue,
                        log::Level::Trace => crossterm::style::Color::White,
                    }),
                    crossterm::style::Print(record.level().to_string()),
                    crossterm::style::ResetColor,
                    crossterm::style::Print(format!(" {} - {}", record.target(), record.args())),
                )
            }
        }
    }
}

use log::{Log, Metadata, Record};

impl Log for Document {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if let Err(error) = self.record(Expression::Log(record)) {
            eprintln!("unable to record log: {}", error);
        }
    }

    fn flush(&self) {}
}

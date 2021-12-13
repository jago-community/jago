#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Poisoned")]
    Poisoned,
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
}

use std::io::{stderr, Stderr, Write};

#[derive(Debug)]
pub struct Document {
    buffer: Vec<u8>,
    steps: Vec<usize>,
    output: Stderr,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            buffer: vec![],
            steps: vec![],
            output: stderr(),
        }
    }
}

impl Write for Document {
    fn write(&mut self, input: &[u8]) -> std::io::Result<usize> {
        let step = self.output.write(input)?;

        self.buffer.extend_from_slice(&input[0..step]);
        self.steps.push(step);

        Ok(step)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.output.flush()
    }
}

impl Document {
    pub fn record(&mut self, expression: Expression) -> Result<(), Error> {
        use crossterm::execute;

        execute!(self, &expression).map_err(Error::from)
    }
}

pub enum Expression<'a> {
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

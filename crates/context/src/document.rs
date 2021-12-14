#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Poisoned")]
    Poisoned,
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
    #[error("NoThingAtPosition ({0},{1})")]
    NoThingAtPosition(usize, usize),
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

        self.buffer.extend_from_slice(&input[..step]);
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

        execute!(self, &expression)?;

        let ranges = expression.get_ranges();

        match expression.kind {
            ExpressionKind::Log(log) => {
                if log.args().to_string().ends_with('?') {
                    println!("is question");
                }
            }
        };

        /*
        if self.is_question() {
            if let Some(answer) = self.read_input()? {
                log::info!("{}", String::from_utf8_lossy(answer));
            }
        }
         */

        Ok(())
    }

    pub fn is_question(&self) -> bool {
        if let Some(part) = self
            .buffer
            .get(self.buffer.len() - 2..self.buffer.len() - 1)
        {
            if let Ok(part) = std::str::from_utf8(part) {
                part.trim_end().ends_with("?")
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn read_input(&mut self) -> Result<Option<&[u8]>, Error> {
        use crossterm::{
            cursor::{
                position, MoveDown, MoveLeft, MoveRight, MoveUp, RestorePosition, SavePosition,
            },
            event::{read, Event, KeyCode, KeyEvent},
            execute,
            terminal::{disable_raw_mode, enable_raw_mode},
        };

        execute!(self, SavePosition, MoveUp(1))?;

        let mut start = 0;
        let mut stop = 0;

        enable_raw_mode()?;

        loop {
            let event = read()?;

            match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }) => {
                    let (x, y) = position()?;

                    if let Ok(buffer) = std::str::from_utf8(self.buffer.as_ref()) {
                        let location =
                            buffer
                                .lines()
                                .enumerate()
                                .fold(0, |mut cursor, (index, line)| {
                                    if index == y as usize {
                                        cursor + x as usize
                                    } else {
                                        cursor += line.len() + 1;
                                        cursor
                                    }
                                });

                        start = location;
                        stop = location + 1;
                    }

                    break;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                }) => {
                    break;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('h'),
                    ..
                }) => {
                    execute!(self, MoveLeft(1))?;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('l'),
                    ..
                }) => {
                    execute!(self, MoveRight(1))?;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('j'),
                    ..
                }) => {
                    execute!(self, MoveDown(1))?;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('k'),
                    ..
                }) => {
                    execute!(self, MoveUp(1))?;
                }
                _ => {}
            };
        }

        disable_raw_mode()?;

        execute!(self, RestorePosition)?;

        Ok(self.buffer.get(start..stop))
    }
}

use std::{cell::RefCell, ops::Range};

pub struct Expression<'a> {
    kind: ExpressionKind<'a>,
    arrangement: RefCell<Vec<Box<dyn Display>>>,
    ranges: RefCell<Vec<(Range<usize>, bool)>>,
    value: RefCell<Vec<u8>>,
}

pub enum ExpressionKind<'a> {
    Log(&'a log::Record<'a>),
}

impl<'a> From<&'a log::Record<'a>> for Expression<'a> {
    fn from(record: &'a log::Record<'a>) -> Self {
        Expression {
            kind: ExpressionKind::Log(record),
            arrangement: Default::default(),
            ranges: Default::default(),
            value: Default::default(),
        }
    }
}

use std::fmt::Display;

impl Expression<'_> {
    pub fn get_ranges(&self) -> Vec<Range<usize>> {
        match self.kind {
            ExpressionKind::Log(log) => {
                let mut ranges = vec![];

                let mut start = 0;

                for line in log.args().to_string().lines() {
                    let end = start + line.len();

                    ranges.push(start..end);

                    start = end + 1;
                }

                ranges
            }
        }
    }

    fn arrange(&self) {
        match self.kind {
            ExpressionKind::Log(record) => {
                use log::Level;

                *self.arrangement.borrow_mut() = vec![
                    Box::new(SetForegroundColor(match record.level() {
                        Level::Error => Color::Red,
                        Level::Warn => Color::Yellow,
                        Level::Info => Color::Green,
                        Level::Debug => Color::Blue,
                        Level::Trace => Color::White,
                    })),
                    Box::new(Print(record.level().to_string())),
                    Box::new(SetForegroundColor(Color::Blue)),
                    Box::new(Print(format!(" {}", record.target()))),
                    Box::new(ResetColor),
                    Box::new(Print(format!(" {}", record.args()))),
                    Box::new(SetForegroundColor(Color::DarkGrey)),
                    Box::new(Print(match (record.file(), record.line()) {
                        (Some(file), Some(line)) => format!(" {}:{}", file, line),
                        _ => "".to_string(),
                    })),
                    Box::new(ResetColor),
                    Box::new(Print("\n")),
                ];

                let (ranges, value): (Vec<(Range<usize>, bool)>, Vec<u8>) =
                    self.arrangement.borrow().iter().enumerate().fold(
                        (vec![], vec![]),
                        |(mut ranges, mut value), (index, block)| {
                            let block = format!("{}", block);

                            let start = value.len();
                            value.extend_from_slice(block.as_bytes());
                            let stop = value.len();

                            ranges.push((start..stop, index == 6));

                            (ranges, value)
                        },
                    );

                *self.ranges.borrow_mut() = self
                    .arrangement
                    .borrow()
                    .iter()
                    .enumerate()
                    .map(|(index, _)| (0..0, index == 6))
                    .collect();

                *self.value.borrow_mut() = vec![];
            }
        }
    }
}

impl std::fmt::Display for Expression<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ExpressionKind::Log(log) => {
                write!(f, "{} {} - {}", log.level(), log.target(), log.args())
            }
        }
    }
}

use crossterm::{
    style::{Color, Print, ResetColor, SetForegroundColor},
    Command,
};

impl<'a> Command for Expression<'a> {
    fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self.kind {
            ExpressionKind::Log(record) => {
                use log::Level;

                let blocks: Vec<Box<dyn std::fmt::Display>> = vec![
                    Box::new(SetForegroundColor(match record.level() {
                        Level::Error => Color::Red,
                        Level::Warn => Color::Yellow,
                        Level::Info => Color::Green,
                        Level::Debug => Color::Blue,
                        Level::Trace => Color::White,
                    })),
                    Box::new(Print(record.level().to_string())),
                    Box::new(SetForegroundColor(Color::Blue)),
                    Box::new(Print(format!(" {}", record.target()))),
                    Box::new(ResetColor),
                    Box::new(Print(format!(" {}", record.args()))),
                    Box::new(SetForegroundColor(Color::DarkGrey)),
                    Box::new(Print(match (record.file(), record.line()) {
                        (Some(file), Some(line)) => format!(" {}:{}", file, line),
                        _ => "".to_string(),
                    })),
                    Box::new(ResetColor),
                    Box::new(Print("\n")),
                ];

                for block in blocks {
                    f.write_fmt(format_args!("{}", block))?;
                }

                Ok(())
            }
        }
    }
}

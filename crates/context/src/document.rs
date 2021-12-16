mod grid;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Poisoned")]
    Poisoned,
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
    #[error("NoThingAtPosition ({0},{1})")]
    NoThingAtPosition(usize, usize),
}

use std::{
    io::{stderr, Stderr, Write},
    ops::Range,
};

#[derive(Debug)]
pub struct Document {
    buffer: Vec<u8>,
    steps: Vec<usize>,
    blocks: Vec<(Range<usize>, bool)>,
    output: Stderr,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            buffer: vec![],
            steps: vec![],
            output: stderr(),
            blocks: vec![],
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

        expression.arrange();

        self.blocks.extend_from_slice(&expression.ranges());

        execute!(self, &expression)?;

        Ok(())
    }

    pub fn read_input(&mut self) -> Result<Option<&[u8]>, Error> {
        use crossterm::{
            cursor::{
                position, MoveDown, MoveLeft, MoveRight, MoveTo, MoveUp, RestorePosition,
                SavePosition,
            },
            event::{read, Event, KeyCode, KeyEvent},
            execute,
            terminal::{disable_raw_mode, enable_raw_mode},
        };

        let (x, y) = self.previous_block_boundary(position()?)?;

        execute!(self, SavePosition, MoveTo(x, y))?;

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

/*
#[test]
fn test_previous_block_boundary() {
    let messages = [
        "gathering",
        "context yes or no?",
        "why things are the way they are",
        "528.453µs elapsed",
    ];

    let mut document = Document::default();

    for message in messages {
        document
            .record(Expression::from(
                &log::Record::builder()
                    .target("nope")
                    .args(format_args!("{}", message))
                    .level(log::Level::Info)
                    .file(Some("src/lib.rs"))
                    .line(Some(0))
                    .build(),
            ))
            .unwrap();
    }

    println!("{}", String::from_utf8_lossy(&document.buffer));

    let output = String::from_utf8_lossy(&document.buffer);

    let last_line_length = output.lines().rev().next().unwrap().len();

    assert_eq!(
        (last_line_length as u16 - 1, 3),
        document.previous_block_boundary((0, 4)).unwrap()
    );
}
*/

impl Document {
    //             1113151719212325272931333537394143454749515355
    //  01234567891012141618202224262830323436384042444648505254
    // 0INFO jago gathering src/lib.rs:113                      33
    // 1INFO context yes or no? crates/context/src/lib.rs:50    84
    // 2INFO jago why things are the way they are src/lib.rs:187139
    // 3INFO jago 528.453µs elapsed src/lib.rs:49               179
    //                            ^
    // 4*                                                       180
    // 10..18, 46..55, 31..124, 149..165
    // * (0,  4) = 33 + 1 + 51 + 1 + 55 + 1 + 40 + 1 = 183
    // ^ (26, 3) = 33 + 1 + 51 + 1 + 55 + 1 + 26 = 168
    fn previous_relevant_position(&self, (_, y): (u16, u16)) -> Result<(u16, u16), Error> {
        unimplemented!()
    }

    fn previous_block_boundary(&self, (_, y): (u16, u16)) -> Result<(u16, u16), Error> {
        let mut blocks = self.blocks.iter().rev().peekable();

        let mut mark = self.buffer.len() - 1;

        dbg!(&mark);

        let mut values = (0..mark)
            .rev()
            .map(|index| (index, &self.buffer[index]))
            .peekable();

        let mut dy = 0;

        loop {
            dbg!((blocks.peek(), values.peek()));

            match (blocks.peek(), values.next()) {
                (Some((range, relevant)), Some((index, _)))
                    if *relevant && range.start > index && range.end < index =>
                {
                    mark = index;
                    dbg!("here");
                    break;
                }
                (Some(_), Some((_, b'\n'))) => dy += 1,
                (Some(block), Some((index, _))) if block.0.start > index => drop(blocks.next()),
                (Some(_), Some(_)) => {}
                _ => {
                    dbg!("there");
                    break;
                }
            };
        }

        dbg!(&mark);

        let mut column = 0;

        for mark in (0..mark).rev() {
            column += 1;

            if self.buffer[mark] == b'\n' {
                break;
            }
        }

        Ok((column, y - dy))
    }
}

use std::cell::RefCell;

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
    pub fn ranges(&self) -> Vec<(Range<usize>, bool)> {
        self.ranges.take()
    }

    fn arrange(&self) {
        if self.value.borrow().len() > 0 {
            return;
        }

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

                *self.ranges.borrow_mut() = ranges;

                *self.value.borrow_mut() = value;
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
        let buffer = self.value.borrow();

        let value = std::str::from_utf8(&buffer).map_err(|_| std::fmt::Error)?;

        f.write_str(value)
    }
}

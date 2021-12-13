use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
enum Encoding {
    UTF8,
}

#[derive(Debug, Clone)]
struct Part {
    size: usize,
    color: Option<Color>,
}

#[derive(Default, Debug, Clone)]
pub struct State {
    buffer: String,
    output: String,
    parts: Vec<Part>,
    step: usize,
    cursor: usize,
}

//use tokio::sync::mpsc;

#[derive(Debug, Default)]
pub struct Context {
    state: Arc<Mutex<State>>,
    //sender: mpsc::UnboundedSender<Part>,
    //receiver: mpsc::UnboundedReceiver<Part>,
}

//impl Default for Context {
//fn default() -> Self {
//let (sender, receiver) = mpsc::unbounded_channel();

//Self {
//sender,
//receiver,
//..Default::default()
//}
//}
//}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("MutexPoisoned")]
    MutexPoisoned,
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
    #[error("SetLogger")]
    SetLogger,
}

use crossterm::style::Color;

use std::io::Write;

use tokio_stream::{Stream, StreamExt};

impl Context {
    fn record(&self, input: &str, color: Option<Color>) {
        if let Ok(mut state) = self.state.lock() {
            state.output.push_str(input);

            state.parts.push(Part {
                size: input.len(),
                color,
            });
        } else {
            eprintln!("unable to record");
        }

        /*
        if let Err(error) = self.sender.send(Part {
            size: input.len(),
            color,
        }) {
            eprintln!("unable to send part {}", error);
        }
        */
    }

    /*
    pub fn document(&'static self) -> Result<(), Error> {
        use tokio::runtime::Runtime;

        let runtime = Runtime::new()?;

        runtime.spawn(async move {
            //self.receiver.for_each(|part| {}).await;

            //for part in self.receiver {
            //// ...
            //}

            while let Some(part) = self.receiver.next().await {
                // ...
            }

            /*loop {
                /*
                let mut state = match self.state.try_lock() {
                    Ok(state) => state,
                    Err(_error) => {
                        eprintln!("unable to obtain state lock");
                        return;
                    }
                };

                println!("here");

                if let Some(part) = state.parts.get(state.step) {
                    dbg!(part);

                    if let Err(error) = execute(&state.output, &part, std::io::stdout()) {
                        eprintln!("error interacting with terminal: {}", error);
                    }
                    state.step += 1;
                }
                */
            }*/
        });

        Ok(())
    }
        */

    pub fn write(&self, input: &str) -> Result<(), Error> {
        let mut guard = self.state.lock().map_err(|_| Error::MutexPoisoned)?;

        guard.buffer.push_str(input);

        Ok(())
    }

    pub fn target(&self) -> Vec<u8> {
        if let Ok(guard) = self.state.lock() {
            Vec::with_capacity(guard.buffer.len())
        } else {
            Vec::new()
        }
    }

    pub fn read(&self, mut target: impl Write) -> Result<(), Error> {
        let guard = self.state.lock().map_err(|_| Error::MutexPoisoned)?;

        target.write_all(&guard.buffer.as_bytes())?;

        Ok(())
    }
}

impl From<String> for Context {
    fn from(input: String) -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                buffer: input,
                ..Default::default()
            })),
            //..Default::default()
        }
    }
}

use log::{Level, Log, Metadata, Record};

impl Log for Context {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        use crossterm::{
            cursor::MoveLeft,
            queue,
            style::{Print, ResetColor, SetForegroundColor},
        };
        use std::io::stdout;

        let mut count = 0u16;

        let mut output = stdout();

        let level = record.level();

        let message = format!("{}", record.level());

        self.record(
            &message,
            Some(match level {
                Level::Error => Color::Red,
                Level::Warn => Color::Yellow,
                Level::Info => Color::Green,
                Level::Debug => Color::Blue,
                Level::Trace => Color::Magenta,
            }),
        );
        self.record(" ", None);

        let attempt = queue!(
            &output,
            SetForegroundColor(match level {
                Level::Error => Color::Red,
                Level::Warn => Color::Yellow,
                Level::Info => Color::Green,
                Level::Debug => Color::Blue,
                Level::Trace => Color::Magenta,
            }),
            Print(&message),
            Print(" "),
            ResetColor
        );

        if let Err(error) = attempt {
            eprintln!("error logging {}", error);
        } else {
            count += 1 + message.len() as u16;
        }

        let message = format!("{}", record.args());

        self.record(&message, None);

        let attempt = queue!(&output, Print(&message));

        let mut expects_answer = false;

        if let Err(error) = attempt {
            eprintln!("error logging {}", error);
        } else {
            if message.ends_with('?') {
                expects_answer = true;
            }

            count += message.len() as u16;
        }

        let mut message = String::new();

        if let Some(file) = record.file() {
            message.push_str(file);
        }

        if let Some(line) = record.line() {
            message.push(' ');
            message.push_str(&format!("{}", line));
        }

        self.record("   ", None);
        self.record(&message, Some(Color::DarkGrey));

        let attempt = queue!(
            &output,
            SetForegroundColor(Color::DarkGrey),
            Print("   "),
            Print(&message),
            ResetColor
        );

        if let Err(error) = attempt {
            eprintln!("error logging {}", error);
        } else {
            count += 3 + message.len() as u16;
        }

        self.record("\n", None);

        let attempt = queue!(&output, Print("\n".to_string()), MoveLeft(count + 1));

        if let Err(error) = attempt {
            eprintln!("error logging {}", error);
        }

        if let Err(error) = output.flush() {
            eprintln!("error logging {}", error);
        }

        /*
        if expects_answer {
            let mut input = String::new();

            if let Err(error) = std::io::stdin().read_line(&mut input) {
                eprintln!("error logging {}", error);
            } else {
                log::info!("{}", input.trim());
            }
        }*/
    }

    //fn flush(&self) {
    //if let Ok(state) = self.state.lock() {
    //let mut cursor = 0;

    //for part in &state.parts {
    //print!("{}", &state.buffer[cursor..cursor + part.size]);
    //cursor += part.size;
    //}
    //}
    //}

    fn flush(&self) {
        use crossterm::{
            cursor::MoveLeft,
            queue,
            style::{Print, ResetColor, SetForegroundColor},
        };

        if let Ok(state) = self.state.lock() {
            let mut output = std::io::stdout();

            let mut cursor = 0;
            let mut line_length = 0;

            for part in &state.parts {
                if let Some(color) = part.color {
                    let _ = queue!(&mut output, SetForegroundColor(color));
                }

                let item = &state.output[cursor..cursor + part.size];

                let _ = queue!(&mut output, Print(item));

                cursor += part.size;

                if part.color.is_some() {
                    let _ = queue!(&mut output, ResetColor);
                }

                if item == "\n" {
                    let _ = queue!(&mut output, MoveLeft(line_length as u16));
                    line_length = 0;
                } else {
                    line_length += part.size;
                }
            }

            if let Err(error) = output.flush() {
                eprintln!("error logging {}", error);
            }
        }
    }
}

fn execute(source: &str, part: &Part, mut target: impl Write) -> Result<(), Error> {
    dbg!(part);

    use crossterm::{
        cursor::MoveLeft,
        queue,
        style::{Print, ResetColor, SetForegroundColor},
    };

    if let Some(color) = part.color {
        let _ = queue!(target, SetForegroundColor(color));
    }

    let item = &source[0..part.size];

    let _ = queue!(target, Print(item));

    //cursor += part.size;

    if part.color.is_some() {
        let _ = queue!(target, ResetColor);
    }

    //if item == "\n" {
    //let _ = queue!(&mut target, MoveLeft(line_length as u16));
    //line_length = 0;
    //} else {
    //line_length += part.size;
    //}

    Ok(())
}

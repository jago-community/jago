use crdts::List;

#[derive(Clone)]
pub struct Context {
    pub buffer: List<u8, u8>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            buffer: List::new(),
        }
    }

    pub fn read_buffer(&self) -> Vec<u8> {
        self.buffer.clone().read_into()
    }
}

impl From<Vec<u8>> for Context {
    fn from(input: Vec<u8>) -> Self {
        let buffer = List::new();

        for byte in input {
            buffer.append(byte, 0);
        }

        Self { buffer }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
}

use std::iter::Peekable;

pub fn handle(
    input: &mut Peekable<impl Iterator<Item = String>>,
    _context: &Context,
) -> Result<(), Error> {
    match input.peek() {
        Some(name) if name == "log" => {
            drop(input.next());

            log::info!("{}", input.collect::<Vec<_>>().join(" "));

            Ok(())
        }
        _ => Ok(()),
    }
}

use log::{Level, Metadata, Record};

impl log::Log for Context {
    fn enabled(&self, metadata: &Metadata) -> bool {
        println!("enabled {:?}", metadata);
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        //if self.enabled(record.metadata()) {
        println!("{} - {}", record.level(), record.args());
        //}
    }

    fn flush(&self) {
        println!("flush");
    }
}

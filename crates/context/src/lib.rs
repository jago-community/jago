#[derive(Default)]
pub struct Inner(Vec<u8>);

impl std::ops::Deref for Inner {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Inner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

use std::sync::{Arc, Mutex};

pub type Context = Arc<Mutex<Inner>>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("Poison {0}")]
    Poison(Box<dyn std::error::Error + 'static>),
}

/*

impl Context {
    pub fn get<'a>(&'static self) -> Result<impl std::ops::Deref<Target = [u8]>, Error> {
        //pub fn get<'a>(&'static self) -> Result<std::sync::MutexGuard<'a, &'a [u8]>, Error> {
        self.0
            .lock()
            //.map(|payload| payload.as_ref())
            .map_err(|error| Error::Poison(Box::new(error)))
    }
}

*/

impl From<Vec<u8>> for Inner {
    fn from(vec: Vec<u8>) -> Self {
        Inner(vec)
    }
}

use std::iter::Peekable;

pub fn handle(
    input: &mut Peekable<impl Iterator<Item = String>>,
    _context: Context,
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

impl log::Log for Inner {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {
        println!("flush");
    }
}

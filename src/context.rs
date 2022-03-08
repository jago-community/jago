#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

use ::{
    crdts::{CmRDT, List},
    crossterm::{cursor::MoveToNextLine, event::Event, style::Print, Command},
    once_cell::sync::OnceCell,
    std::{
        fmt,
        sync::{Arc, Mutex},
    },
};

use crate::{Directives, Handle};

pub struct Context {
    inner: Arc<Mutex<Inner>>,
}

impl Handle for Context {
    fn handle(&self, event: &Event) -> Directives {
        match self.inner.lock() {
            Ok(mut inner) => {
                inner.apply(*event);

                Directives::empty()
            }
            Err(error) => {
                log::error!("context::handle inner lock: {}", error);

                Directives::STOP
            }
        }
    }
}

struct Inner {
    buffer: List<char, u8>,
}

impl Context {
    pub fn get(buffer: &str) -> &'static Self {
        static CONTEXT: OnceCell<Context> = OnceCell::new();

        CONTEXT.get_or_init(move || Context::from(buffer))
    }
}

static DEFAULT_ACTOR: u8 = 0;

impl From<&str> for Context {
    fn from(input: &str) -> Self {
        let mut buffer = List::new();

        for c in input.chars() {
            let op = buffer.append(c, DEFAULT_ACTOR);
            buffer.apply(op);
        }

        Self {
            inner: Arc::new(Mutex::new(Inner { buffer })),
        }
    }
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let buffer = self
            .inner
            .lock()
            .map_err(|_| fmt::Error)?
            .buffer
            .read::<String>();

        f.write_str(&buffer)
    }
}

impl Command for Context {
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        let inner = self.inner.lock().map_err(|_| fmt::Error)?;

        for c in inner.buffer.iter() {
            if c == &'\n' {
                MoveToNextLine(1).write_ansi(out)?;
            } else {
                Print(*c).write_ansi(out)?;
            }
        }

        Ok(())
    }
}

impl CmRDT for Inner {
    type Op = Event;
    type Validation = Error;

    fn validate_op(&self, _: &Self::Op) -> Result<(), Self::Validation> {
        Ok(())
    }

    fn apply(&mut self, op: Self::Op) {
        match op {
            _ => {}
        }
    }
}

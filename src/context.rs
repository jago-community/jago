#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("Lock")]
    Lock,
    #[error("SetLogger {0}")]
    SetLogger(#[from] log::SetLoggerError),
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

use ::{
    crdts::{CmRDT, List, MVReg},
    std::sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct Context {
    logs: Arc<Mutex<List<char, u8>>>,
    cursor: Arc<Mutex<MVReg<usize, u8>>>,
    state: Arc<Mutex<MVReg<State, u8>>>,
}

#[derive(Clone, PartialEq)]
pub enum State {
    Continue,
    Done(Option<i32>),
}

impl State {
    fn stop(&self) -> bool {
        match self {
            State::Done(_) => true,
            _ => false,
        }
    }
}

use ::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

impl CmRDT for State {
    type Op = Event;
    type Validation = Error;

    fn validate_op(&self, _: &Self::Op) -> Result<(), Self::Validation> {
        Ok(())
    }

    fn apply(&mut self, op: Self::Op) {
        match op {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
            }) => {
                *self = State::Done(None);
            }
            _ => {}
        }
    }
}

use ::{
    crossterm::{
        cursor::{Hide, MoveTo, Show},
        event::EventStream,
        execute,
        terminal::{
            disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
            LeaveAlternateScreen,
        },
    },
    futures::{
        future,
        stream::{self, StreamExt},
    },
    std::{
        io::{stdout, Write},
        ops::Deref,
    },
    tokio::runtime,
};

use crate::grid::CharGrid;

impl Context {
    pub fn watch(&self) -> Result<(), Error> {
        let runtime = runtime::Builder::new_current_thread().build()?;

        runtime.block_on(async move {
            let reader = EventStream::new();

            let (x, y) = size()?;

            if let State::Done(_) = self.handle(Event::Resize(x, y)) {
                return Ok(());
            }

            let mut out = stdout();

            execute!(&out, EnterAlternateScreen, Hide)?;

            enable_raw_mode()?;

            let (_, _) = reader
                .flat_map(|result| stream::iter(result.ok()))
                .map(|event| self.handle(event))
                .map(|state| self.render().map(|result| (state, result)))
                .flat_map(|maybe| stream::iter(maybe))
                .flat_map(|(state, grid)| {
                    stream::iter(
                        execute!(&mut out, Clear(ClearType::All), MoveTo(0, 0), grid)
                            .map(|_| state)
                            .map_err(Error::from)
                            .ok(),
                    )
                })
                .filter(|state: &State| future::ready(state.stop()))
                .into_future()
                .await;

            disable_raw_mode()?;

            let mut out = stdout();

            execute!(&out, LeaveAlternateScreen, Show)?;

            out.flush()?;

            Ok(())
        })
    }

    fn handle(&self, event: Event) -> State {
        log::info!("handling event {:?}", event);

        if let Ok(state) = self.state.lock() {
            let read = state.read();

            if let Some(current) = read.val.first() {
                let mut next = current.clone();

                next.apply(event);

                if next.stop() {
                    return next;
                }

                let write = read.derive_add_ctx(0);

                state.write(next.clone(), write);

                next
            } else {
                State::Continue
            }
        } else {
            State::Continue
        }
    }

    fn render<'a>(&'a self) -> Option<CharGrid> {
        self.logs
            .lock()
            .map(|list| list.deref().iter().cloned().collect::<Vec<_>>())
            .map(CharGrid::new)
            .ok()
    }
}

use ::once_cell::sync::OnceCell;

impl Context {
    pub fn get() -> Result<&'static Self, Error> {
        static CONTEXT: OnceCell<Context> = OnceCell::new();

        let context = CONTEXT.get_or_init(|| Context {
            logs: Arc::new(Mutex::new(List::new())),
            cursor: Arc::new(Mutex::new(MVReg::new())),
            state: Arc::new(Mutex::new({
                let mut state = MVReg::new();
                let write = state.read_ctx().derive_add_ctx(0);
                let op = state.write(State::Continue, write);
                state.apply(op);
                state
            })),
        });

        log::set_logger(context)?;
        log::set_max_level(log::LevelFilter::Info);

        Ok(context)
    }
}

use log::{Log, Metadata, Record};

impl Log for Context {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &Record<'_>) {
        if let Ok(mut logs) = self.logs.lock() {
            for ch in format!(
                "{} {} {:?}\n",
                record.level(),
                record.args(),
                record
                    .file()
                    .and_then(|file| { record.line().map(|line| (file, line)) })
            )
            .chars()
            {
                let op = logs.append(ch, 0);

                logs.apply(op);
            }
        }
    }

    fn flush(&self) {
        if let Ok(logs) = self.logs.lock() {
            if let Ok(mut cursor) = self.cursor.lock() {
                let read = cursor.read_ctx();
                let op = cursor.write(logs.len(), read.derive_add_ctx(0));
                cursor.apply(op);
            }
        }
    }
}

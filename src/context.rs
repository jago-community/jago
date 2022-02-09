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

impl CmRDT for Context {
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
                if let Ok(mut state) = self.state.lock() {
                    let read = state.read_ctx();
                    let write = read.derive_add_ctx(0);
                    let op = state.write(State::Done(None), write);
                    state.apply(op);
                }
            }
            _ => {}
        }
    }
}

use ::{
    crossterm::{
        cursor::{Hide, Show},
        event::EventStream,
        execute, queue,
        terminal::{
            disable_raw_mode, enable_raw_mode, size, EnterAlternateScreen, LeaveAlternateScreen,
        },
    },
    futures::{
        future,
        stream::{self, StreamExt},
    },
    std::io::{stdout, Write},
    tokio::runtime,
};

impl Context {
    pub fn watch(&self) -> Result<(), Error> {
        let runtime = runtime::Builder::new_current_thread().build()?;

        runtime.block_on(async move {
            let reader = EventStream::new();

            let (x, y) = size()?;

            self.apply(Event::Resize(x, y));

            if let Ok(state) = self.state.lock() {
                let read = state.read();

                if let Some(current) = read.val.first() {
                    if current.stop() {
                        return Ok(());
                    }
                }
            }

            let mut out = stdout();

            execute!(&out, EnterAlternateScreen, Hide)?;

            enable_raw_mode()?;

            reader
                .flat_map(|item| stream::iter(item.ok()))
                //.map(|event| handle.handle_event(&event))
                //.take_while(|op| future::ready(!op.stop()))
                .for_each(move |_| {
                    //before_result.expect("good bye");

                    // queue!(&mut out, &view).expect("hello");

                    out.flush().expect("gah");

                    future::ready(())
                })
                .await;

            disable_raw_mode()?;

            let mut out = stdout();

            execute!(&out, LeaveAlternateScreen, Show)?;

            out.flush()?;

            Ok(())
        })
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
        println!("flush");

        if let Ok(logs) = self.logs.lock() {
            if let Ok(mut cursor) = self.cursor.lock() {
                let read = cursor.read_ctx();
                let op = cursor.write(logs.len(), read.derive_add_ctx(0));
                cursor.apply(op);
            }
        }
    }
}

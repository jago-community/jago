use ::{
    crdts::{CmRDT, List},
    std::sync::{Arc, Mutex},
};

pub struct Context {
    logs: Arc<Mutex<List<char, u8>>>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("Lock")]
    Lock,
    #[error("SetLogger {0}")]
    SetLogger(#[from] log::SetLoggerError),
}

use ::once_cell::sync::OnceCell;

impl Context {
    pub fn get() -> Result<&'static Self, Error> {
        static CONTEXT: OnceCell<Context> = OnceCell::new();

        let context = CONTEXT.get_or_init(|| Context {
            logs: Arc::new(Mutex::new(List::new())),
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
            format!(
                "{} {} {:?}\n",
                record.level(),
                record.args(),
                record
                    .file()
                    .and_then(|file| { record.line().map(|line| (file, line)) })
            )
            .chars()
            .map(|ch| logs.append(ch, 0))
            .for_each(|op| logs.apply(op));
        }
    }

    fn flush(&self) {}
}
use std::sync::{Arc, Mutex};

use crdts::{CmRDT, List, MVReg};

pub struct Logger {
    logs: Arc<Mutex<List<char, u8>>>,
    gate: Arc<Mutex<MVReg<usize, u8>>>,
}

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

impl Logger {
    pub fn get() -> Result<&'static Self, Error> {
        use once_cell::sync::OnceCell;

        static LOGGER: OnceCell<Logger> = OnceCell::new();

        let logger = LOGGER.get_or_init(|| Logger {
            logs: Arc::new(Mutex::new(List::new())),
            gate: Arc::new(Mutex::new(MVReg::new())),
        });

        log::set_logger(logger)?;
        log::set_max_level(log::LevelFilter::Info);

        Ok(logger)
    }
}

use log::{Log, Metadata, Record};

impl Log for Logger {
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
            if let Ok(mut gate) = self.gate.lock() {
                let read = gate.read_ctx();
                let op = gate.write(logs.len(), read.derive_add_ctx(0));
                gate.apply(op);
            }
        }
    }
}

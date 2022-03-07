pub fn before() -> Result<(), Error> {
    let logger = Logger::get()?;

    log::set_logger(logger)?;
    log::set_max_level(log::LevelFilter::Info);

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("NoHome")]
    NoHome,
    #[error("SetLogger {0}")]
    SetLogger(#[from] log::SetLoggerError),
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

use ::{
    once_cell::sync::OnceCell,
    std::{
        fs::{create_dir_all, OpenOptions},
        io::{stdout, Stdout, Write},
        path::PathBuf,
        sync::{Arc, Mutex},
    },
};

pub struct Logger {
    out: Stdout,
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl Logger {
    fn get() -> Result<&'static Self, Error> {
        static LOGGER: OnceCell<Logger> = OnceCell::new();

        let logger = LOGGER.get_or_init(move || Logger {
            out: stdout(),
            buffer: Default::default(),
        });

        Ok(logger)
    }
}

fn target_dir() -> Result<PathBuf, Error> {
    let target = dirs::home_dir()
        .map_or(Err(Error::NoHome), Ok)
        .map(|home| home.join("jago").join("target").join("jago").join("logs"))?;

    Ok(target)
}

use log::{Log, Metadata, Record};

impl Log for Logger {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &Record<'_>) {
        let log = format!(
            "{} {} {:?}\n",
            record.level(),
            record.args(),
            record
                .file()
                .and_then(|file| { record.line().map(|line| (file, line)) })
        );

        let mut handle = self.out.lock();

        if let Err(error) = handle.write_all(log.as_bytes()) {
            eprintln!("error writing logs to stdout: {}", error);
        }

        if let Ok(mut buffer) = self.buffer.lock() {
            if let Err(error) = buffer.write_all(log.as_bytes()) {
                eprintln!("error writing logs to stdout: {}", error);
            }
        }

        self.flush();
    }

    fn flush(&self) {
        static TARGET: OnceCell<Result<PathBuf, Error>> = OnceCell::new();

        let target = TARGET.get_or_init(move || {
            let target = target_dir()?;

            create_dir_all(&target)?;

            Ok(target)
        });

        if let Ok(target) = target {
            if let Ok(mut file) = OpenOptions::new().append(true).open(target.join("log")) {
                if let Ok(buffer) = self.buffer.lock() {
                    if let Err(error) = file.write_all(buffer.as_ref()) {
                        eprintln!("error writing logs to stdout: {}", error);
                    }
                }
            }
        }
    }
}

use std::{iter::Peekable, path::PathBuf};

pub fn before() -> Result<Box<dyn Fn()>, Error> {
    let output_directory = std::env::var("OUTPUT_DIRECTORY")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::home_dir().unwrap().join("output"));

    let logger = flexi_logger::Logger::with_str("info")
        .log_to_file()
        .buffer_and_flush()
        .directory(&output_directory)
        .append()
        .rotate(
            flexi_logger::Criterion::Size(10 * 1024 * 1024),
            flexi_logger::Naming::Numbers,
            flexi_logger::Cleanup::KeepLogFiles(5),
        )
        .print_message()
        .start()?;

    Ok(Box::new(move || {
        logger.shutdown();
    }))
}

pub fn handle<I: Iterator<Item = String>>(input: &mut Peekable<I>) -> Result<(), Error> {
    dbg!(1);
    match dbg!(input.peek()) {
        Some(next) if next == "log" => input.next(),
        _ => return Err(Error::Incomplete),
    };
    dbg!(2);
    let level = if let Some(first) = input.next() {
        first
    } else {
        return Err(Error::Incomplete);
    };
    dbg!(3);
    let level = level.parse()?;

    let rest = input.collect::<Vec<_>>().join(" ");
    log::log!(level, "{}", rest);

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Incomplete,
    Logger(flexi_logger::FlexiLoggerError),
    Level(log::ParseLevelError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Incomplete => write!(f, "incomplete"),
            Self::Logger(error) => write!(f, "{}", error),
            Self::Level(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Incomplete => None,
            Self::Logger(error) => Some(error),
            Self::Level(error) => Some(error),
        }
    }
}

impl From<flexi_logger::FlexiLoggerError> for Error {
    fn from(error: flexi_logger::FlexiLoggerError) -> Self {
        Self::Logger(error)
    }
}

impl From<log::ParseLevelError> for Error {
    fn from(error: log::ParseLevelError) -> Self {
        Self::Level(error)
    }
}

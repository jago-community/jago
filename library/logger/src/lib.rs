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
    match input.peek() {
        Some(next) if next == "log" => input.next(),
        _ => return Err(Error::Incomplete),
    };

    let level = if let Some(first) = input.next() {
        first
    } else {
        return Err(Error::Incomplete);
    };

    let level = level.parse()?;

    let rest = input.collect::<Vec<_>>().join(" ");
    log::log!(level, "{}", rest);

    Ok(())
}

author::error!(
    Incomplete,
    flexi_logger::FlexiLoggerError,
    log::ParseLevelError,
);

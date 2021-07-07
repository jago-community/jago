use std::{iter::Peekable, path::PathBuf};

pub fn before() -> Result<Box<dyn Fn()>, Error> {
    let logger_directory = std::env::var("CARGO_MANIFEST_DIR")?;

    let output_directory = PathBuf::from(&logger_directory)
        .join("target")
        .join("logger");

    let libraries = library::libraries(&logger_directory)?;

    let libraries_filter = libraries.join("=debug,");

    let filter = format!("warn, {}", libraries_filter);

    let logger = flexi_logger::Logger::with_str(&filter)
        .log_to_file()
        .print_message()
        .buffer_and_flush()
        .directory(&output_directory)
        .append()
        .rotate(
            flexi_logger::Criterion::Size(10 * 1024 * 1024),
            flexi_logger::Naming::Numbers,
            flexi_logger::Cleanup::KeepLogFiles(5),
        )
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
    std::env::VarError,
    library::Error,
);

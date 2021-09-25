use std::iter::Peekable;

pub fn before() -> Result<Option<Box<dyn Fn()>>, Error> {
    let logger_directory = std::env::var("CARGO_MANIFEST_DIR")?;

    let concepts = encyclopedia::concepts(&logger_directory)?;

    let concept_filter = concepts.join("=debug,");

    let filter = format!("warn, {}", concept_filter);

    pretty_env_logger::formatted_builder()
        .parse_filters(&filter)
        .init();

    Ok(None)
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

book::error!(
    Incomplete,
    flexi_logger::FlexiLoggerError,
    log::ParseLevelError,
    std::env::VarError,
    encyclopedia::Error,
);
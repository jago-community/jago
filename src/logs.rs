#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Setup(#[from] log::SetLoggerError),
}

#[cfg(target_os = "android")]
pub fn before() -> Result<(), Error> {
    android_logger::init_once(
        android_logger::Config::default()
            .with_min_level(log::Level::Trace)
            .with_tag("jago"),
    );

    Ok(())
}

#[cfg(not(target_os = "android"))]
pub fn before() -> Result<(), Error> {
    pretty_env_logger::formatted_builder()
        .filter_module("jago", log::LevelFilter::Info)
        .filter_module("interface", log::LevelFilter::Info)
        .try_init()
        .map_err(Error::from)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(not(target_arch = "wasm32"))]
    #[error("SetLogger {0}")]
    SetLogger(#[from] log::SetLoggerError),
    #[cfg(not(target_arch = "wasm32"))]
    #[error("SetLogger {0}")]
    Environment(#[from] environment::Error),
    #[cfg(not(target_arch = "wasm32"))]
    #[error("Io {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(target_arch = "wasm32")]
pub fn before() -> Result<(), Error> {
    use log::Level;

    console_log::init_with_level(Level::Info);

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn before() -> Result<(), Error> {
    use ::{
        simplelog::{
            ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
        },
        std::fs::OpenOptions,
    };

    let path = environment::target("logs", true).map(|path| path.join("current"))?;

    let file = OpenOptions::new().create(true).append(true).open(path)?;

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Warn,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(LevelFilter::Info, Config::default(), file),
    ])
    .map_err(Error::from)
}

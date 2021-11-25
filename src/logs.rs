use std::io::Write;

use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};

pub fn configure() -> Result<(), Error> {
    let config_dir = dirs::config_dir()
        .map(|path| path.join("jago"))
        .map_or(Err(Error::NoConfig), Ok)?;

    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
    }

    let config = config_dir.join("logs.yml");

    if !config.exists() {
        let config_bytes = br#"# Scan this file for changes every 30 seconds
refresh_rate: 30 seconds

appenders:
  # An appender named "stdout" that writes to stdout
  stdout:
    kind: console

  pipe:
    kind: file
    path: "target/pipe.log"
    encoder:
      pattern: "{d} - {m}{n}"

# Set the default logging level to "warn" and attach the "stdout" appender to the root
root:
  level: warn
  appenders:
    - stdout

loggers:
  jago:
    level: info
    appenders:
      - pipe
    additive: false"#;

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&config)?;

        file.write_all(config_bytes)?;
    }

    log4rs::init_file(&config, Default::default()).map_err(Error::from)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("NoConfig")]
    NoConfig,
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
    #[error("LogSetup {0}")]
    LogSetup(#[from] anyhow::Error),
}

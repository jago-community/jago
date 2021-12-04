#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Setup(#[from] log::SetLoggerError),
    #[error("{0}")]
    Context(#[from] context::Error),
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

use context::Context;

#[cfg(not(target_os = "android"))]
pub fn before(context: &'static Context) -> Result<(), Error> {
    /*
    pretty_env_logger::formatted_builder()
        .filter_module("jago", log::LevelFilter::Info)
        .filter_module("interface", log::LevelFilter::Info)
        .filter_module("handle", log::LevelFilter::Info)
        .filter_module("workspace", log::LevelFilter::Info)
        .try_init()
        .map_err(Error::from)
    */

    unsafe {
        log::set_logger_racy(context).expect("set logger");
    }

    Ok(())
}

//use log::{Level, Metadata, Record};

//struct SimpleLogger;

//impl log::Log for SimpleLogger {
//fn enabled(&self, metadata: &Metadata) -> bool {
//metadata.level() <= Level::Info
//}

//fn log(&self, record: &Record) {
//if self.enabled(record.metadata()) {
//println!("{} - {}", record.level(), record.args());
//}
//}

//fn flush(&self) {}
//}

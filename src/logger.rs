pub fn before() -> Result<(), Error> {
    use ::{
        env_logger::{fmt::Color, Builder},
        log::{Level, LevelFilter},
        std::io::Write,
    };

    let mut builder = Builder::from_default_env();

    builder
        .format(|buf, record| {
            let level = record.level();

            let mut level_style = buf.style();

            level_style
                .set_color(match level {
                    Level::Trace => Color::Blue,
                    Level::Debug => Color::Magenta,
                    Level::Info => Color::Green,
                    Level::Warn => Color::Yellow,
                    Level::Error => Color::Red,
                })
                .set_bold(true);

            write!(
                buf,
                "{} {}",
                level_style.value(record.level()),
                record.args()
            )?;

            let mut file_style = buf.style();

            file_style.set_color(Color::Ansi256(245)).set_bold(true);

            if let Some(file) = record.file() {
                write!(buf, " {}{}", file_style.value(file), file_style.value(":"))?;
            }

            if let Some(line) = record.line() {
                write!(buf, "{}", file_style.value(line))?;
            }

            writeln!(buf, "")
        })
        .filter(None, LevelFilter::Info)
        .init();

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("SetLogger {0}")]
    SetLogger(#[from] log::SetLoggerError),
}

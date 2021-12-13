mod context;
pub mod document;
mod pipe;

pub use context::{Context, Error};
pub use document::Document;

use once_cell::sync::OnceCell;

static DOCUMENT: OnceCell<Document> = OnceCell::new();

pub fn before() -> Result<(), Error> {
    use crossterm::terminal::enable_raw_mode;

    let document = DOCUMENT.get_or_init(Document::default);

    log::set_logger(document).map_err(|_| Error::SetLogger)?;
    log::set_max_level(log::LevelFilter::Info);

    //enable_raw_mode().map_err(Error::from)

    Ok(())
}

use log::Log;

pub fn after() -> Result<(), Error> {
    use crossterm::terminal::disable_raw_mode;

    if let Some(document) = DOCUMENT.get() {
        document.flush();
    }

    //context.flush();

    // disable_raw_mode().map_err(Error::from)

    Ok(())
}

use std::iter::Peekable;

pub fn handle(
    input: &mut Peekable<impl Iterator<Item = String>>,
    _context: &Context,
) -> Result<(), Error> {
    match input.peek() {
        Some(name) if name == "log" => {
            drop(input.next());

            log::info!("{}", input.collect::<Vec<_>>().join(" "));

            Ok(())
        }
        _ => Ok(()),
    }
}

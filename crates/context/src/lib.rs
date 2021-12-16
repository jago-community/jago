mod context;
pub mod document;
//mod parse;

pub use crate::context::{Context, Error};

use once_cell::sync::OnceCell;

static CONTEXT: OnceCell<Context> = OnceCell::new();

use log::LevelFilter;

pub fn before() -> Result<(), Error> {
    use crossterm::terminal::enable_raw_mode;

    let context = CONTEXT.get_or_init(Context::default);

    log::set_logger(context).map_err(|_| Error::SetLogger)?;
    log::set_max_level(LevelFilter::Info);

    //enable_raw_mode().map_err(Error::from)

    Ok(())
}

use log::Log;

pub fn after() -> Result<(), Error> {
    use crossterm::terminal::disable_raw_mode;

    let context = CONTEXT.get().ok_or(Error::AfterBefore)?;

    context.flush();

    //disable_raw_mode().map_err(Error::from)

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

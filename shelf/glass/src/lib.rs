mod life;
//mod video;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("Life {0}")]
    Life(#[from] life::Error),
    //#[error("Life {0}")]
    //Video(#[from] video::Error),
}

use context::Context;
use std::iter::Peekable;

pub fn handle(
    input: &mut Peekable<impl Iterator<Item = String>>,
    _context: &mut Context,
) -> Result<(), Error> {
    match input.peek() {
        Some(next) if &next == &"glass" => {
            drop(input.next());

            life::handle().map_err(Error::from)
        }
        _ => Ok(()),
    }

    //video::handle().map_err(Error::from)
}

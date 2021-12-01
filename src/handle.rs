use context::Context;

use std::{iter::Peekable, path::PathBuf};

pub fn grasp<'a>(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &'a mut Context,
) -> Result<(), Error> {
    match input.peek().map(PathBuf::from) {
        Some(next)
            if next.exists()
                && next.ends_with(PathBuf::from("NativeMessagingHosts").join("jago.json")) =>
        {
            crate::pipe::handle(input, context).map_err(Error::from)
        }
        _ => interface::handle(
            &mut ["interface".to_string()]
                .iter()
                .cloned()
                .chain(input)
                .peekable(),
            context,
        )
        .map_err(Error::from),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("Watch {0}")]
    Interface(#[from] interface::Error),
    #[error("Pipe {0}")]
    Pipe(#[from] crate::pipe::Error),
}

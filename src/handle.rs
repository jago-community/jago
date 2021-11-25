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
            log::info!("is native host message");
            crate::pipe::handle(input, context).map_err(Error::from)
        }
        _ => watch::handle(
            &mut ["watch".to_string()]
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
    Watch(#[from] watch::Error),
    #[error("Pipe {0}")]
    Pipe(#[from] crate::pipe::Error),
}

use std::iter::Peekable;

use crate::Context;

pub fn handle(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &mut Context,
) -> Result<(), crate::Error> {
    if let Ok(rep) = String::from_utf8(context.to_vec()) {
        let _difference = std::mem::replace(context, rep.to_uppercase().as_bytes().to_vec());
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Incomplete")]
    Incomplete,
}

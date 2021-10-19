use std::iter::Peekable;

use crate::Context;

pub fn handle(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &mut Context,
) -> Result<(), Error> {
    if let Ok(rep) = String::from_utf8(context.to_vec()) {
        let _difference = std::mem::replace(context, rep.to_uppercase().as_bytes().to_vec());
    }

    match input.peek() {
        Some(next) if &next[..] == "pack" => pack(input, context),
        _ => Err(Error::Incomplete),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
}

fn pack(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &mut Context,
) -> Result<(), Error> {
    //unimplemented!()
    Ok(())
}
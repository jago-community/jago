use context::Context;

use std::iter::Peekable;

pub fn handle<'a>(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &'a mut Context,
) -> Result<(), Error> {
    match input.peek() {
        Some(name) if name == "jago" => {
            let _ = input.next();
        }
        _ => {}
    };

    let _difference = replace(context, b"why things are the way they are".to_vec());

    /// With regards to the computer science, I think the problem is calling it a memory leak.
    /// It's misleading in my opinion. I mean I didn't get the joke until I properly learned a low
    /// level programming language. I know I'm not the smartest guy but I can't be the only one that
    /// missed the point.
    ///
    /// Why not call it what it is?
    ///
    /// Unchecked growth.
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
}

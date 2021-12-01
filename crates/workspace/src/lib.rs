use context::Context;

use std::{iter::Peekable, mem::replace};

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
    ///
    /// But is it actually? Or just the natural progression of things. In case I'm right in
    /// guessing that I'm not the only one that missed the punchline I'm going to try to explain it
    /// here as simply as possible.
    ///
    /// To start, we are at present moving into what people in the know have coined the unbang. I
    /// used the foo foo-y phrase "we are at present" as an artistic transition into the graph I'll
    /// show you next.
    ///
    /// > Someone turned off the internet.
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("NoHome")]
    NoHome,
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
    #[error("NoResourceDirectory")]
    NoResourceDirectory,
}

use std::path::PathBuf;

pub fn source_directory() -> Result<PathBuf, Error> {
    dirs::home_dir().map_or(Err(Error::NoHome), |home| Ok(home.join("jago")))
}

pub fn resource_directory() -> Result<PathBuf, Error> {
    let mut current = std::env::current_exe()?;

    loop {
        if !current.pop() {
            return Err(Error::NoResourceDirectory);
        }

        if current.join("Resources").exists() {
            return Ok(current.join("Resources"));
        } else if current.join("assets").exists() {
            return Ok(current);
        }
    }
}

use context::Context;

use std::{
    io::{stdin, stdout, Read, Write},
    iter::Peekable,
    path::PathBuf,
};

use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};

pub fn handle(
    input: &mut Peekable<impl Iterator<Item = String>>,
    context: &mut Context,
) -> Result<(), Error> {
    match input.peek() {
        Some(next) if PathBuf::from(next).exists() => {}
        _ => return Ok(()),
    };

    let arguments = input.collect::<Vec<_>>();

    let mut input = stdin();

    loop {
        let length = input.read_u32::<NativeEndian>()?;

        let mut buffer = vec![0; length as usize];
        input.read_exact(&mut buffer)?;

        let message: &str = serde_json::from_slice(&buffer)?;

        let output = serde_json::to_vec(&format!(
            "arguments {:?}\nmessage: {:?}\ncontext: {:?}",
            arguments,
            message,
            std::str::from_utf8(context)
        ))?;

        let mut out = stdout();
        out.write_u32::<NativeEndian>(output.len() as u32)?;
        out.write_all(&output)?;
        out.flush()?;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
    #[error("InputOutput {0}")]
    InputOutput(#[from] std::io::Error),
    #[error("JavaScriptObjectNotation {0}")]
    JavaScriptObjectNotation(#[from] serde_json::Error),
}

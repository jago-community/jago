book::error!(Incomplete, std::io::Error, serde_json::Error);

use std::{
    io::{stdin, stdout, Read, Write},
    iter::Peekable,
};

use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};

pub fn handle<I: Iterator<Item = String>>(_input: &mut Peekable<I>) -> Result<(), Error> {
    let mut input = stdin();

    let mut context = puzzle::Puzzle::empty();

    loop {
        let length = input.read_u32::<NativeEndian>()?;

        let mut buffer = vec![0; length as usize];
        input.read_exact(&mut buffer)?;

        let (_setting, key): (u8, String) = serde_json::from_slice(&buffer)?;

        context.wrap(key);

        let output = serde_json::to_vec(&format!("{:?}", context))?;

        let mut out = stdout();
        out.write_u32::<NativeEndian>(output.len() as u32)?;
        out.write_all(&output)?;
        out.flush()?;
    }
}

#[derive(serde::Serialize)]
struct Output {
    key: String,
}

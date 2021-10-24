book::error!(
    Incomplete,
    std::io::Error,
    serde_json::Error,
    //encyclopedia::Error
);

use std::{
    io::{stdin, stdout, Read, Write},
    iter::Peekable,
};

use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};

use crdts::merkle_reg::Node;

pub fn handle<I: Iterator<Item = String>>(_input: &mut Peekable<I>) -> Result<(), Error> {
    let mut input = stdin();

    //let mut context = puzzle::Puzzle::empty();

    loop {
        let length = input.read_u32::<NativeEndian>()?;

        let mut buffer = vec![0; length as usize];
        input.read_exact(&mut buffer)?;

        let (_setting, key): (u8, Node<Vec<u8>>) = serde_json::from_slice(&buffer)?;

        // TODO; leftoff with trouble encoding/decoding merkle nodes

        // let node: Node<Vec<u8>> = serde_json::from_str(&key)?;

        // context.wrap(key);

        // let output = encyclopedia::handle(&context)?;

        // let output = serde_json::to_vec(&output)?;

        let output = serde_json::to_vec(&format!("hello {:?}", key))?;

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

use crdts::{CmRDT, List};

pub struct Buffer {
    data: List<u8, u8>,
    output: Vec<u8>,
}

use std::io::{self, Write};

impl Write for Buffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let a = bytes
            .iter()
            .map(|byte| self.data.append(*byte, 0))
            .fold(0, |size, op| {
                self.data.apply(op);
                size + 1
            });

        Ok(a)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

use crdts::{CmRDT, List};

pub struct Document {
    source: List<char, u8>,
}

impl<I: IntoIterator<Item = char>> From<I> for Document {
    fn from(iter: I) -> Self {
        let mut source = List::new();

        for chunk in iter {
            let op = source.append(chunk, 0);
            source.apply(op);
        }

        Self { source }
    }
}

use ::{
    crossterm::{cursor::MoveToNextLine, style::Print, QueueableCommand},
    std::io::{self, Write},
};

impl Document {
    pub fn queue_ansi<W: QueueableCommand + Write>(&self, buffer: &mut W) -> io::Result<()> {
        for c in self.source.iter() {
            if c == &'\n' {
                buffer.queue(MoveToNextLine(1))?;
            } else {
                buffer.queue(Print(*c))?;
            }
        }

        Ok(())
    }
}

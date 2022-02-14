#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
}

use crdts::{CmRDT, MVReg};

pub struct Cursor {
    xy: MVReg<(usize, usize), u8>,
}

use crossterm::event::{Event, KeyCode, KeyEvent};

impl CmRDT for Cursor {
    type Op = Event;
    type Validation = Error;

    fn validate_op(&self, _: &Self::Op) -> Result<(), Self::Validation> {
        Ok(())
    }

    fn apply(&mut self, op: Self::Op) {
        match op {
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                ..
            }) => {
                let read = self.xy.read();

                if let Some(next) = read
                    .val
                    .first()
                    .iter()
                    .flat_map(|(x, y)| y.checked_add(1).map(|dy| (*x, dy)))
                    .next()
                {
                    let op = self.xy.write(next, read.derive_add_ctx(0));

                    self.xy.apply(op);
                }
            }
            _ => {}
        }
    }
}

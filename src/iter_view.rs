pub struct IterView<Item, Inner>(Inner, Option<Item>, usize);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
}

impl<'a, Item, Inner: Iterator<Item = Item>> From<Inner> for IterView<Item, Inner> {
    fn from(inner: Inner) -> Self {
        Self(inner, None, 0)
    }
}

use std::fmt;

impl<'a, Item, Inner: Iterator<Item = Item>> IterView<Item, Inner> {
    pub fn step(&mut self) {
        if let Some(next) = self.2.checked_add(1) {
            self.2 = next;
            self.1 = self.0.next();
        }
    }
}

use crossterm::{cursor::MoveToColumn, style::Print, Command};

impl<'a, Item: fmt::Debug, Inner: Iterator<Item = Item>> Command for IterView<Item, Inner> {
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        Print(format!("{:?}", self.1))
            .write_ansi(out)
            .and(Print("\n\n").write_ansi(out))
            .and(MoveToColumn(3).write_ansi(out))
            .and(Print(format!("step {}", self.2)).write_ansi(out))
    }
}

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

impl<'a, Item, Inner: Iterator<Item = Item>> IterView<Item, Inner> {
    pub fn handle(&mut self, event: &Event) -> Result<(), Error> {
        let mut stop = false;

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('n'),
                ..
            }) => {
                if self.2 > 0 && self.1.is_none() {
                    stop = true;
                } else {
                    self.step();
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => {
                stop = true;
            }
            _ => {}
        };

        if stop {
            Err(Error::Incomplete)
        } else {
            Ok(())
        }
    }
}

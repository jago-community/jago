use crdts::{CmRDT, List};

pub struct Buffer {
    data: List<char, u8>,
}

impl<Set: AsRef<str>> From<Set> for Buffer {
    fn from(set: Set) -> Self {
        let mut data = List::new();
        for item in set.as_ref().chars() {
            data.apply(data.append(item, 0));
        }
        Self { data }
    }
}

use crate::directives::{Command, Directive, Event, KeyCode, KeyEvent, Op};

impl Directive for Buffer {
    fn handle(&mut self, event: &Event) -> Op {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char(code),
                ..
            }) => self.append(*code),
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => self.append('\n'),
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            }) if self.data.len() > 0 => {
                log::info!("{}", self.data.len());

                self.delete(self.data.len() - 1)
            }
            _ => self.handle_common(event),
        }
    }
}

impl Buffer {
    fn append(&mut self, item: char) -> Op {
        self.data.apply(self.data.append(item, 0));

        Op::Continue
    }

    fn delete(&mut self, index: usize) -> Op {
        if let Some(op) = self.data.delete_index(index, 0) {
            self.data.apply(op)
        }

        Op::Continue
    }
}

use ::{
    crossterm::{cursor::MoveToNextLine, style::Print},
    itertools::{FoldWhile, Itertools},
    std::fmt,
};

impl Command for Buffer {
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        self.data
            .iter()
            .map(|item| {
                if item == &'\n' {
                    MoveToNextLine(0).write_ansi(out)
                } else {
                    Print(item).write_ansi(out)
                }
            })
            .fold_while(Ok(()), |_, next| {
                if next.is_ok() {
                    FoldWhile::Continue(Ok(()))
                } else {
                    FoldWhile::Done(next)
                }
            })
            .into_inner()
    }
}

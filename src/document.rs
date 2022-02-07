use crdts::{CmRDT, Dot, List, MVReg};

type Actor = u8;

pub struct Document {
    units: List<char, Actor>,
    cursor: MVReg<Cursor, Actor>,
    clock: Dot<Actor>,
}

use std::fmt::Debug;

pub trait Span: Debug {
    fn width(&self) -> usize;
}

impl<T: Span> Span for &T {
    fn width(&self) -> usize {
        (**self).width()
    }
}

impl<A: Span> Span for &[A] {
    fn width(&self) -> usize {
        self.iter().map(|span| (*span).width()).sum()
    }
}

impl Span for str {
    fn width(&self) -> usize {
        self.len()
    }
}

impl Span for char {
    fn width(&self) -> usize {
        self.len_utf8()
    }
}

impl Span for std::str::Chars<'_> {
    fn width(&self) -> usize {
        self.clone().map(|span| span.width()).sum()
    }
}

#[derive(Default, Clone, Debug, PartialEq, PartialOrd)]
pub struct Cursor(usize, (usize, usize));

#[test]
fn cursor() {
    let document = include_str!("../poems/etheridge-knight/haiku/1");

    let cursor = Cursor::default();

    let cursor = cursor
        .find((1, 1), document.chars().collect_vec().as_ref(), |u| {
            u == &'\n'
        })
        .unwrap();

    assert_eq!(
        cursor,
        Cursor(
            "Eastern guard tower
g"
            .len(),
            (1, 1)
        )
    );

    let cursor = cursor
        .find((0, 0), document.chars().collect_vec().as_ref(), |u| {
            u == &'\n'
        })
        .unwrap();

    assert_eq!(cursor, Cursor(0, (0, 0)));
}

use itertools::{FoldWhile, Itertools};

impl Cursor {
    fn find<'a, U>(
        &self,
        (dx, dy): (isize, isize),
        buffer: &[U],
        split: impl Fn(&U) -> bool,
    ) -> Option<Self>
    where
        U: 'a + Span,
    {
        let ax = dx.unsigned_abs();
        let ay = dy.unsigned_abs();

        buffer
            .into_iter()
            // batching adjust for y
            // batching adjust for x
            .scan((0usize, 0usize, 0usize), |(x, y, z), next| {
                *z += next.width();

                if split(next) {
                    *y = if dy.is_positive() {
                        y.checked_add(next.width())?
                    } else {
                        y.checked_sub(next.width())?
                    };

                    *x = 0;
                } else {
                    *x = if dx.is_positive() {
                        x.checked_add(next.width())?
                    } else {
                        x.checked_sub(next.width())?
                    };
                }

                Some(Cursor(*z, (*x, *y)))
            })
            .find(|Cursor(_, (x, y))| y == &ay && x == &ax)
    }
}

impl Document {
    fn cursor(&self) -> Cursor {
        let read = self.cursor.read();

        self.get_cursor(&read.val)
    }

    fn get_cursor(&self, read: &[Cursor]) -> Cursor {
        read.first().cloned().unwrap_or_default()
    }
}

impl<Set: AsRef<str>> From<Set> for Document {
    fn from(set: Set) -> Self {
        let mut units = List::new();
        for item in set.as_ref().chars() {
            units.apply(units.append(item, 0));
        }
        Self {
            units,
            cursor: Default::default(),
            clock: Dot::new(0, 0),
        }
    }
}

use crossterm::{style::Print, Command};

use std::fmt;

impl Command for Document {
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        self.units
            .iter()
            .map(|unit| Print(unit).write_ansi(out))
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

#[derive(PartialEq)]
pub enum Operation {
    Continue,
    Done,
    Exit(Option<i32>),
    // Cursor
    Step((isize, isize)),
}

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

impl Document {
    pub fn handle(&mut self, event: &Event) -> Operation {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => Operation::Exit(None),
            Event::Key(KeyEvent {
                code: KeyCode::Char('h'),
                ..
            }) => Operation::Step((-1, 0)),
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                ..
            }) => Operation::Step((0, 1)),
            Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                ..
            }) => Operation::Step((0, -1)),
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                ..
            }) => Operation::Step((1, 0)),
            _ => Operation::Continue,
        }
    }

    fn apply(&mut self, operation: Operation) -> Operation {
        let next = match operation {
            Operation::Step(d) => self.step(d),
            _ => operation,
        };

        next
    }

    fn step(&mut self, (dx, dy): (isize, isize)) -> Operation {
        let read_cursor = self.cursor.read();

        let read_units: Vec<_> = self.units.read();

        if let Some(next) = self
            .get_cursor(&read_cursor.val)
            .find((dx, dy), &*read_units, |u| u == &&'\n')
        {
            let read = self.cursor.read();

            self.cursor
                .apply(self.cursor.write(next, read.derive_add_ctx(0)));
        }

        Operation::Continue
    }
}

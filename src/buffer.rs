use crdts::{CmRDT, List};

pub struct Buffer {
    mode: Mode,
    bytes: List<u8, u8>,
    cursor: (usize, (usize, usize)),
    sequence: Vec<char>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            bytes: List::new(),
            sequence: vec![],
            cursor: Default::default(),
            mode: Default::default(),
        }
    }
}

impl Buffer {
    pub fn read_bytes(&self) -> Vec<u8> {
        self.bytes.iter().cloned().collect()
    }
}

#[derive(Debug, PartialEq)]
enum Mode {
    Cursor,
    Edit,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Cursor
    }
}

impl<'a> From<&'a [u8]> for Buffer {
    fn from(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes.iter().fold(List::new(), |mut list, byte| {
                list.apply(list.append(*byte, 0));
                list
            }),
            ..Default::default()
        }
    }
}

use crossterm::{
    cursor::{CursorShape, MoveTo, SetCursorShape},
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    Command,
};

use crate::{
    color::ColorPicker,
    slice::{Reference, Slice},
};

impl Command for Buffer {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        let slice = self.read_bytes();

        let slice = Slice::from(&slice[..]);

        Clear(ClearType::All).write_ansi(out)?;
        MoveTo(0, 0).write_ansi(out)?;

        let mut references = slice.graphemes();

        let mut color_picker = ColorPicker::new();

        while let Some(reference) = references.next() {
            SetForegroundColor(color_picker.pick()).write_ansi(out)?;

            let (x, y) = reference.coordinates();
            // TODO: only do this for the one after a new line.
            MoveTo(x as u16, y as u16).write_ansi(out)?;

            if let Some(grapheme) = slice.get(reference) {
                Print(grapheme).write_ansi(out)?;
            }
        }

        SetForegroundColor(Color::Green).write_ansi(out)?;
        Print(format!(
            "\n\n{:?} {:?}\n\n{:?}\nfactor {:?} -> {}",
            Reference::from(self.cursor).layout(),
            slice.get(Reference::from(self.cursor)),
            &self.mode,
            &self.sequence,
            factor(&self.sequence).1
        ))
        .write_ansi(out)?;

        SetForegroundColor(Color::Magenta).write_ansi(out)?;
        Print("\n\nq ").write_ansi(out)?;
        ResetColor.write_ansi(out)?;
        Print("= quit").write_ansi(out)?;
        SetForegroundColor(Color::Magenta).write_ansi(out)?;
        Print("\nh, j, k, l ").write_ansi(out)?;
        ResetColor.write_ansi(out)?;
        Print("= left, down, up, right").write_ansi(out)?;
        SetForegroundColor(Color::Magenta).write_ansi(out)?;
        Print("\nControl+n ").write_ansi(out)?;
        ResetColor.write_ansi(out)?;
        Print("= Change poem.").write_ansi(out)?;
        SetForegroundColor(Color::Magenta).write_ansi(out)?;
        Print("\n{a:some number}").write_ansi(out)?;
        SetForegroundColor(Color::Blue).write_ansi(out)?;
        Print("{b:some direction key}").write_ansi(out)?;
        ResetColor.write_ansi(out)?;
        Print(" = Move in the ").write_ansi(out)?;
        SetForegroundColor(Color::Blue).write_ansi(out)?;
        Print("{b}").write_ansi(out)?;
        ResetColor.write_ansi(out)?;
        Print(" direction ").write_ansi(out)?;
        SetForegroundColor(Color::Magenta).write_ansi(out)?;
        Print("{a}").write_ansi(out)?;
        ResetColor.write_ansi(out)?;
        Print(" times.\n").write_ansi(out)?;

        let (x, y) = Reference::from(self.cursor).coordinates();

        MoveTo(x as u16, y as u16).write_ansi(out)?;

        match self.mode {
            Mode::Cursor => {
                SetCursorShape(CursorShape::UnderScore).write_ansi(out)?;
            }
            Mode::Edit => {
                SetCursorShape(CursorShape::Line).write_ansi(out)?;
            }
        };

        Ok(())
    }
}

fn factor(sequence: &[char]) -> (usize, usize) {
    let got = sequence
        .iter()
        .enumerate()
        .map(|(index, maybe_digit)| (index + 1, maybe_digit.to_digit(10)))
        .take_while(|(_, result)| result.is_some())
        .map(|(index, result)| (index, result.unwrap()))
        .fold((0, 0), |(_, factor), (index, digit)| {
            (index, factor * 10 + digit)
        });

    (got.0 as usize, if got.0 == 0 { 1 } else { got.1 as usize })
}

impl Buffer {
    fn consume_factor(&mut self) -> usize {
        let got = factor(&self.sequence);

        self.sequence = self.sequence.drain(got.0..).collect();

        got.1
    }
}

#[test]
fn test_factor() {
    let sequences = vec![
        (vec!['1', '0', '2'], 3, 102),
        (vec!['2'], 1, 2),
        (vec![], 0, 1),
        (vec!['b'], 0, 1),
    ];

    for (sequence, want_took, want_factor) in sequences {
        let mut buffer = Buffer {
            sequence: sequence.clone(),
            ..Default::default()
        };

        assert_eq!(buffer.consume_factor(), want_factor);

        assert_eq!(&buffer.sequence[..], &sequence[want_took..]);
    }
}

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

impl Buffer {
    pub fn handle(&mut self, slice: Slice, event: &Event) -> bool {
        let mut next = self.cursor.clone();

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('h'),
                ..
            }) if self.mode == Mode::Cursor => {
                let factor = self.consume_factor();

                let mut references = slice.graphemes_before(self.cursor.into()).skip(factor - 1);

                if let Some(next_ref) = references.next() {
                    next = (next_ref.index(), next_ref.coordinates());
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                ..
            }) if self.mode == Mode::Cursor => {
                let factor = self.consume_factor();

                let mut references = slice
                    .closest_x_in_y_after(self.cursor.into())
                    .skip(factor - 1);

                if let Some(next_ref) = references.next() {
                    next = (next_ref.index(), next_ref.coordinates());
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                ..
            }) if self.mode == Mode::Cursor => {
                let factor = self.consume_factor();

                let mut references = slice
                    .closest_x_in_y_before(self.cursor.into())
                    .skip(factor - 1);

                if let Some(next_ref) = references.next() {
                    next = (next_ref.index(), next_ref.coordinates());
                } else {
                    return true;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                ..
            }) if self.mode == Mode::Cursor => {
                let factor = self.consume_factor();

                let mut references = slice.graphemes_after(self.cursor.into()).skip(factor - 1);

                if let Some(next_ref) = references.next() {
                    next = (next_ref.index(), next_ref.coordinates());
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('i'),
                ..
            }) if self.mode == Mode::Cursor => {
                self.mode = Mode::Edit;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('a'),
                ..
            }) if self.mode == Mode::Cursor => {
                let mut references = slice.graphemes_after(self.cursor.into());

                if let Some(next_ref) = references.next() {
                    next = (next_ref.index(), next_ref.coordinates());
                }

                self.mode = Mode::Edit;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(' '),
                modifiers,
            }) if self.mode == Mode::Edit && modifiers.contains(KeyModifiers::CONTROL) => {
                self.mode = Mode::Cursor;

                let mut references = slice.graphemes_before(self.cursor.into());

                if let Some(next_ref) = references.next() {
                    next = (next_ref.index(), next_ref.coordinates());
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(code),
                ..
            }) if self.mode == Mode::Cursor => {
                self.sequence.push(*code);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(code),
                ..
            }) if self.mode == Mode::Edit => {
                self.bytes
                    .apply(self.bytes.insert_index(self.cursor.0, *code as u8, 0));

                let mut references = slice.graphemes_after(self.cursor.into());

                if let Some(next_ref) = references.next() {
                    next = (next_ref.index(), next_ref.coordinates());
                }
            }
            _ => {}
        }

        self.cursor = next;

        false
    }
}

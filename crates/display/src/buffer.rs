use crdts::{CmRDT, List};

pub struct Buffer {
    cursor: usize,
    bytes: List<u8, u8>,
}

impl Buffer {
    pub fn new(from: impl AsRef<[u8]>, actor: u8) -> Buffer {
        let mut list = List::new();

        for byte in from.as_ref() {
            list.apply(list.append(byte, actor));
        }

        Buffer {
            cursor: 0,
            bytes: List::new(),
        }
    }

    pub fn to_string(&self) -> String {
        self.bytes.iter().map(|byte| *byte as char).collect()
    }

    pub fn current_unicode<'a>(&self, buffer: &'a str) -> &'a str {
        &buffer[self.cursor..].graphemes(true).next().unwrap_or("")
    }
}

use crossterm::{
    cursor::{MoveTo, MoveToColumn},
    style::{Color, Print, SetForegroundColor},
    terminal::{Clear, ClearType},
    Command,
};
use itertools::{FoldWhile, Itertools};
use unicode_segmentation::UnicodeSegmentation;

impl Command for Buffer {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        Clear(ClearType::All).write_ansi(out)?;
        MoveTo(0, 0).write_ansi(out)?;

        let current = self.to_string();

        let (mut current_x, mut current_y) = (0, 0);

        let mut color_picker = ColorPicker::new();

        let result = current
            .graphemes(true)
            .into_iter()
            .fold_while(Ok(()), |result, token| {
                if let Err(error) = SetForegroundColor(color_picker.pick()).write_ansi(out) {
                    return FoldWhile::Done(Err(error));
                }

                if let Err(error) = Print(token).write_ansi(out) {
                    return FoldWhile::Done(Err(error));
                }

                if token == "\n" {
                    if let Err(error) = MoveToColumn(0).write_ansi(out) {
                        return FoldWhile::Done(Err(error));
                    }
                }

                FoldWhile::Continue(result)
            })
            .into_inner();

        if let Err(error) = result {
            return Err(error);
        }

        SetForegroundColor(color_picker.pick()).write_ansi(out)?;
        MoveToColumn(0).write_ansi(out)?;
        Print(format!(
            "\n{:?} {:?} {}",
            self.current_unicode(&current),
            (current_x, current_y),
            self.cursor,
        ))
        .write_ansi(out)?;

        MoveTo(current_x as u16, current_y as u16).write_ansi(out)?;

        Ok(())
    }
}

use rand::rngs::ThreadRng;

struct ColorPicker {
    rng: ThreadRng,
    seq: [usize; 231],
}

impl ColorPicker {
    fn new() -> Self {
        let mut seq = [0; 231];

        for i in 0..seq.len() {
            seq[i] = i;
        }

        Self {
            rng: Default::default(),
            seq,
        }
    }
}

use rand::seq::SliceRandom;

impl ColorPicker {
    fn pick(&mut self) -> Color {
        Color::AnsiValue(*self.seq.choose(&mut self.rng).unwrap_or(&231) as u8)
    }
}

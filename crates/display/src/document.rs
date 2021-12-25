#[derive(Default)]
pub struct Document<'a> {
    source: &'a str,
    focus: (u16, u16),
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Machine {0}")]
    Machine(#[from] std::io::Error),
}

impl<'a> Document<'a> {
    pub fn new(source: &'a [u8], focus: (u16, u16)) -> Self {
        Self {
            source: unsafe { std::str::from_utf8_unchecked(source) },
            focus,
        }
    }
}

impl<'a> From<&'a [u8]> for Document<'a> {
    fn from(source: &'a [u8]) -> Self {
        Self {
            source: unsafe { std::str::from_utf8_unchecked(source) },
            ..Default::default()
        }
    }
}

impl<'a> Document<'a> {
    fn color(&self, index: usize, focus: bool) -> u8 {
        if focus {
            231
        } else {
            (index % 230) as u8
        }
    }
}

use unicode_segmentation::UnicodeSegmentation;

impl<'a> crossterm::Command for Document<'a> {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        let (mut x, mut y) = (0, 0);

        for (index, grapheme) in self.source.grapheme_indices(true) {
            let focus = self.focus == (x, y);

            crossterm::style::SetForegroundColor(crossterm::style::Color::AnsiValue(
                self.color(index, focus),
            ))
            .write_ansi(out)?;

            out.write_str(if focus && grapheme == " " {
                "_"
            } else {
                grapheme
            })?;

            if grapheme == "\n" {
                x = 0;
                y += 1;
            } else {
                x += grapheme.len() as u16;
            }
        }

        Ok(())
    }
}

pub struct Buffer<'a> {
    bytes: &'a [u8],
}

impl<'a> Buffer<'a> {
    pub fn new(bytes: &'a [u8]) -> Buffer<'a> {
        Buffer { bytes }
    }

    fn as_str(&self) -> &'a str {
        unsafe { std::str::from_utf8_unchecked(self.bytes) }
    }
}

use crossterm::{
    cursor::{position, MoveTo, MoveToColumn},
    style::{Print, SetForegroundColor},
    terminal::{Clear, ClearType},
    Command,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::color::ColorPicker;

impl<'a> Command for Buffer<'a> {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        let position = position().map_err(|_| std::fmt::Error)?;

        Clear(ClearType::All).write_ansi(out)?;
        MoveTo(0, 0).write_ansi(out)?;

        let current = self.as_str();

        let mut color_picker = ColorPicker::new();

        let result = current
            .graphemes(true)
            .into_iter()
            .fold(Ok((0, 0)), |result, token| {
                if let Err(error) = SetForegroundColor(color_picker.pick()).write_ansi(out) {
                    return Err(error);
                }

                if let Err(error) = Print(token).write_ansi(out) {
                    return Err(error);
                }

                if token == "\n" {
                    if let Err(error) = MoveToColumn(0).write_ansi(out) {
                        return Err(error);
                    }
                }

                result
            });

        if let Err(error) = result {
            return Err(error);
        }

        MoveTo(position.0, position.1).write_ansi(out)?;

        Ok(())
    }
}

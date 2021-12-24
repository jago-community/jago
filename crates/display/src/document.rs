#[derive(Default)]
pub struct Document<'a> {
    source: &'a str,
}

impl<'a> From<&'a [u8]> for Document<'a> {
    fn from(source: &'a [u8]) -> Self {
        Self {
            source: unsafe { std::str::from_utf8_unchecked(source) },
        }
    }
}

use unicode_segmentation::UnicodeSegmentation;

impl<'a> crossterm::Command for Document<'a> {
    fn write_ansi(&self, out: &mut impl std::fmt::Write) -> std::fmt::Result {
        for (index, grapheme) in self.source.graphemes(true).enumerate() {
            crossterm::style::SetForegroundColor(crossterm::style::Color::AnsiValue(
                (index % 230) as u8,
            ))
            .write_ansi(out)?;

            out.write_str(grapheme)?;
        }

        Ok(())
    }
}

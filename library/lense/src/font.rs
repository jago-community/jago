author::error!(
    NotAFont,
    CopyFont,
    MemoryFontNotSupported,
    font_kit::error::SelectionError,
    font_kit::error::FontLoadingError,
    nannou::text::font::Error,
);

use nannou::text::{font, Font};

pub fn get(name: &str) -> Result<Font, Error> {
    let source = font_kit::sources::fs::FsSource::new();

    let handle = source.select_by_postscript_name(name)?;

    let path = match &handle {
        font_kit::handle::Handle::Path { path, .. } => path,
        _ => return Err(Error::MemoryFontNotSupported),
    };

    font::from_file(path).map_err(Error::from)
}

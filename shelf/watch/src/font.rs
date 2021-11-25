use font_kit::{handle::Handle as FontHandle, source::SystemSource};
use rand::seq::SliceRandom;
use wgpu_glyph::ab_glyph::FontArc;

pub struct Cache {
    source: SystemSource,
    fonts: Option<Vec<FontHandle>>,
    fonts_rng: rand::rngs::ThreadRng,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("FontSelection {0}")]
    FontSelection(#[from] font_kit::error::SelectionError),
    #[error("NoFont")]
    NoFont,
    #[error("FontLoad {0}")]
    FontLoading(#[from] font_kit::error::FontLoadingError),
    #[error("NoFontData")]
    NoFontData,
    #[error("InvalidFont")]
    InvalidFont(#[from] wgpu_glyph::ab_glyph::InvalidFont),
}

impl Cache {
    pub fn new() -> Self {
        let mut source = SystemSource::new();

        Self {
            source,
            fonts: None,
            fonts_rng: rand::thread_rng(),
        }
    }

    pub fn pick_font(&mut self) -> Result<FontArc, Error> {
        let fonts = match self.fonts {
            Some(ref fonts) => fonts.clone(),
            _ => {
                let fonts = SystemSource::new().all_fonts()?;
                self.fonts = Some(fonts.clone());
                fonts
            }
        };

        let font = fonts
            .choose(&mut self.fonts_rng)
            .map_or(Err(Error::NoFont), Ok)?;

        let font = font.load()?;
        let font = font.copy_font_data().map_or(Err(Error::NoFontData), Ok)?;

        FontArc::try_from_vec(font.as_ref().clone()).map_err(Error::from)
    }
}

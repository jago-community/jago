use wgpu_glyph::{ab_glyph, GlyphBrush, GlyphBrushBuilder, Section, Text};

author::error!(
    NoFonts,
    CopyFont,
    NoAdaptor,
    ArcGetMut,
    GlyphDraw(String),
    font_kit::error::SelectionError,
    font_kit::error::FontLoadingError,
    wgpu_glyph::ab_glyph::InvalidFont,
);

use std::collections::HashMap;

#[derive(Default)]
pub struct Cache<'a> {
    cache: HashMap<&'a str, Vec<u8>>,
}

impl<'a> Cache<'a> {
    fn get(&'a mut self, key: &'a str) -> Result<GlyphBrush<()>, Error> {
        let raw = self.cache.get(key);

        let font = ab_glyph::FontArc::try_from_vec(raw)?;

        Ok(GlyphBrushBuilder::using_font(font).build(&device, render_format))
    }
}

fn load(key: &str) -> Result<GlyphBrush<()>, Error> {
    unimplemented!()
}

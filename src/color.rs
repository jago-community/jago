use rand::rngs::ThreadRng;

pub struct ColorPicker {
    rng: ThreadRng,
    seq: [usize; 231],
}

impl ColorPicker {
    pub fn new() -> Self {
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

pub use crossterm::style::Color;

impl ColorPicker {
    pub fn pick(&mut self) -> Color {
        Color::AnsiValue(*self.seq.choose(&mut self.rng).unwrap_or(&231) as u8)
    }
}

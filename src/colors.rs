use rand::rngs::ThreadRng;

pub use crossterm::style::Color;

pub struct ColorPicker {
    seq: [usize; 231],
    rng: ThreadRng,
}

impl Default for ColorPicker {
    fn default() -> Self {
        let mut seq = [0; 231];

        for i in 0..seq.len() {
            seq[i] = i;
        }

        Self {
            seq,
            rng: rand::thread_rng(),
        }
    }
}

use rand::seq::SliceRandom;

impl ColorPicker {
    pub fn pick(&self) -> Color {
        let mut rng = self.rng.clone();

        Color::AnsiValue(*self.seq.choose(&mut rng).unwrap_or(&231) as u8)
    }
}

use rand::rngs::ThreadRng;

pub use crossterm::style::Color;

pub struct ColorPicker {
    rng: ThreadRng,
    seq: [usize; 231],
    selected: Option<Color>,
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
            selected: None,
        }
    }
}

use rand::seq::SliceRandom;

impl ColorPicker {
    pub fn pick(&mut self) -> Color {
        Color::AnsiValue(*self.seq.choose(&mut self.rng).unwrap_or(&231) as u8)
    }
}

use super::traits::{Command, Directive, Event, KeyCode, KeyEvent, KeyModifiers, Op};

impl Directive for ColorPicker {
    fn handle(&mut self, event: &Event) -> Op {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('8'),
                modifiers,
            }) if modifiers.contains(KeyModifiers::CONTROL) => {
                self.selected = Some(self.pick());

                Op::Continue
            }
            _ => self.handle_common(event),
        }
    }
}

use ::{crossterm::style::SetForegroundColor, std::fmt};

impl Command for ColorPicker {
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        self.selected
            .map(|selected| SetForegroundColor(selected).write_ansi(out))
            .unwrap_or(Err(fmt::Error))
    }
}

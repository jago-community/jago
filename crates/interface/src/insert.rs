use matches::matches;
use nannou::{
    event::{Event, Key, WindowEvent},
    winit::event_loop::ControlFlow,
};

use crate::model::{Mode, Model};

pub fn event(mode: &Mode, control_flow: &mut ControlFlow, model: &mut Model, event: &Event) {
    if !matches!(model.mode, Mode::Insert) {
        return;
    }

    match event {
        Event::WindowEvent {
            simple: Some(WindowEvent::KeyPressed(Key::Space | Key::C)),
            ..
        } => {
            if model.modifiers.ctrl() {
                model.mode = Mode::Normal;
            }
        }
        _ => {}
    }
}

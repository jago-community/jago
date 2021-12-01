use nannou::{
    event::{Event, Key, ModifiersState, WindowEvent},
    winit::event_loop::ControlFlow,
};

use crate::model::{Mode, Model};

pub fn event(_: &Mode, _: &mut ControlFlow, model: &mut Model, event: &Event) {
    match event {
        Event::WindowEvent {
            simple: Some(WindowEvent::KeyPressed(Key::LControl | Key::RControl)),
            ..
        } => {
            model.modifiers.insert(ModifiersState::CTRL);
        }
        Event::WindowEvent {
            simple: Some(WindowEvent::KeyReleased(Key::LControl | Key::RControl)),
            ..
        } => {
            model.modifiers.remove(ModifiersState::CTRL);
        }
        _ => {}
    }
}

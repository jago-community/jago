use matches::matches;
use nannou::{
    event::{Event, Key, ModifiersState, WindowEvent},
    geom::vec2,
    winit::event_loop::ControlFlow,
};

use crate::model::{Mode, Model};

pub fn event(mode: &Mode, control_flow: &mut ControlFlow, model: &mut Model, event: &Event) {
    if !matches!(mode, &Mode::Normal) {
        return;
    }

    match event {
        Event::WindowEvent {
            simple: Some(WindowEvent::MouseMoved(pos)),
            ..
        } => {
            model.cursor = *pos;
        }
        Event::WindowEvent {
            simple: Some(WindowEvent::Touch(touch)),
            ..
        } => {
            model.cursor = touch.position;
        }
        Event::WindowEvent {
            simple: Some(WindowEvent::KeyPressed(Key::H)),
            ..
        } => {
            model.cursor -= vec2(model.scale, 0.);
        }
        Event::WindowEvent {
            simple: Some(WindowEvent::KeyPressed(Key::L)),
            ..
        } => {
            model.cursor += vec2(model.scale, 0.);
        }
        Event::WindowEvent {
            simple: Some(WindowEvent::KeyPressed(Key::K)),
            ..
        } => {
            model.cursor += vec2(0., model.factor);
        }
        Event::WindowEvent {
            simple: Some(WindowEvent::KeyPressed(Key::J)),
            ..
        } => {
            model.cursor -= vec2(0., model.factor);
        }
        Event::WindowEvent {
            simple: Some(WindowEvent::KeyPressed(Key::I)),
            ..
        } => {
            model.mode = Mode::Insert;
        }
        Event::WindowEvent {
            simple: Some(WindowEvent::KeyPressed(Key::C)),
            ..
        } => {
            if model.modifiers.ctrl() {
                *control_flow = ControlFlow::Exit;
            }
        }
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
            model.modifiers.insert(ModifiersState::CTRL);
        }
        _ => {}
    }
}

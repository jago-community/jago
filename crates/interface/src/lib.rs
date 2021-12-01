use nannou::{prelude::*, winit::event::ModifiersState};

use context::Context;
use std::iter::Peekable;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
}

pub fn handle(
    input: &mut Peekable<impl Iterator<Item = String>>,
    _context: &mut Context,
) -> Result<(), Error> {
    match input.peek() {
        Some(next) if &next == &"interface" => {
            drop(input.next());

            nannou::app(model)
                .update(update)
                .event(event)
                .simple_window(view)
                .run();
        }
        _ => {}
    }

    Ok(())
}

enum Mode {
    Normal,
    Insert,
}

struct Model {
    cursor: Point2,
    scale: f32,
    factor: f32,
    logo: wgpu::Texture,
    mode: Mode,
    modifiers: ModifiersState,
}

fn model(app: &App) -> Model {
    let resources = workspace::resource_directory().unwrap();

    let logo_path = resources.join("assets").join("favicon.ico");

    let logo = wgpu::Texture::from_path(app, logo_path).unwrap();

    Model {
        cursor: pt2(0., 0.),
        scale: 42.,
        factor: 8.,
        logo,
        mode: Mode::Normal,
        modifiers: ModifiersState::empty(),
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn event(app: &App, model: &mut Model, event: Event) {
    match event {
        Event::WindowEvent {
            simple: Some(WindowEvent::MouseMoved(pos)),
            ..
        } => {
            model.cursor = pos;
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
            simple: Some(WindowEvent::KeyPressed(Key::Space)),
            ..
        } => {
            if model.modifiers.ctrl() {
                model.mode = Mode::Normal;
            }
        }
        Event::WindowEvent {
            simple: Some(WindowEvent::KeyPressed(Key::C)),
            ..
        } => {
            if model.modifiers.ctrl() {
                app.quit();
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

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.texture(&model.logo);

    logo(model, &draw);

    let container = app.window_rect().pad(model.factor * 2.);

    draw.text(match model.mode {
        Mode::Insert => "Insert",
        Mode::Normal => "Normal",
    })
    .align_text_bottom()
    .left_justify()
    .color(BLACK)
    .xy(container.xy())
    .wh(container.wh());

    draw.to_frame(app, &frame).unwrap();
}

fn logo(model: &Model, draw: &Draw) {
    draw.background().color(WHITE);

    let bar = Rect::from_x_y_w_h(
        model.cursor.x,
        model.cursor.y,
        model.scale,
        model.factor * 2.,
    )
    .shift_y(1.5 * model.scale);

    draw.rect().xy(bar.xy()).wh(bar.wh()).color(BLUE);

    draw.ellipse()
        .xy(model.cursor)
        .radius(model.scale)
        .color(BLUE);
}

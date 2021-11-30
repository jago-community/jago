use nannou::prelude::*;

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
}

fn model(app: &App) -> Model {
    let assets = app.assets_path().unwrap();
    let img_path = assets.join("favicon.ico");
    let logo = wgpu::Texture::from_path(app, img_path).unwrap();

    Model {
        cursor: pt2(0., 0.),
        scale: 42.,
        factor: 8.,
        logo,
    }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn event(_app: &App, model: &mut Model, event: Event) {
    match event {
        Event::WindowEvent {
            simple: Some(WindowEvent::MouseMoved(pos)),
            ..
        } => {
            model.cursor = pos;
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
        _ => {}
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.texture(&model.logo);

    logo(model, &draw);

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

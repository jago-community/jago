use nannou::prelude::*;

use context::Context;
use std::iter::Peekable;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Incomplete")]
    Incomplete,
}

mod model;

pub fn handle(
    input: &mut Peekable<impl Iterator<Item = String>>,
    _context: &mut Context,
) -> Result<(), Error> {
    match input.peek() {
        Some(next) if &next == &"interface" => {
            drop(input.next());

            nannou::app(model::handle)
                .update(update)
                .event(event)
                .simple_window(view)
                .run();
        }
        _ => {}
    }

    Ok(())
}

use model::{Mode, Model};

fn update(_app: &App, _model: &mut Model, _update: Update) {}

mod all;
mod insert;
mod normal;

use nannou::winit::event_loop::ControlFlow;

fn event(app: &App, model: &mut Model, event: Event) {
    let mut control_flow = ControlFlow::Poll;

    let mode = model.mode.clone();

    //let handles = vec![all::event, insert::event, normal::event];

    for event_handler in [all::event, insert::event, normal::event] {
        event_handler(&mode, &mut control_flow, model, &event);
    }

    match control_flow {
        ControlFlow::Exit => {
            app.quit();
        }
        _ => {}
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.texture(&model.logo);

    logo(model, &draw);

    let container = app.window_rect().pad(model.factor * 2.);

    document(app, container, model, &draw);

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

use matches::matches;

fn document(app: &App, container: Rect, model: &Model, draw: &Draw) {
    let text = text("greetings stranger")
        .left_justify()
        .align_top()
        .font_size(model.scale as u32)
        .build(container);

    draw.path().fill().color(BLACK).events(text.path_events());

    if matches!(model.mode, Mode::Insert)
        && app.elapsed_frames() % (model.scale as u64) < (model.scale / 2.) as u64
    {
        let line_height = text.height();

        let bar = Rect::from_x_y_w_h(
            container.left(),
            container.top() - line_height,
            1.,
            line_height,
        );

        draw.rect().xy(bar.xy()).wh(bar.wh()).color(BLUE);
    }
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

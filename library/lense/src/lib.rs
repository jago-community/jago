mod parts;

use std::iter::Peekable;

use nannou::{
    app::App,
    color,
    event::{Key, Update, WindowEvent},
    frame::Frame,
    text::text,
    window,
};

author::error!(Incomplete, DrawError(String), parts::Error);

pub fn handle<I: Iterator<Item = String>>(_input: &mut Peekable<I>) -> Result<(), Error> {
    let viewer = nannou::app(model).view(view).update(update);
    Ok(viewer.run())
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

struct Model {
    font_size: u32,
    font_color: (f32, f32, f32),
    background_color: (f32, f32, f32),
    sequence: Vec<Key>,
    _window_ids: Vec<window::Id>,
}

fn model(app: &App) -> Model {
    app.new_window()
        .title("Jago Lense")
        .size(680, 480)
        .event(event)
        .view(view)
        .build()
        .unwrap();

    let window_ids = app.window_ids();

    for id in app.window_ids() {
        if let Some(window) = app.window(id) {
            window.set_window_icon(
                parts::logo(
                    color::rgb::Rgb::new(0., 121., 255.),
                    nannou::geom::Rect::from_w_h(16., 24.),
                )
                .ok(),
            );
        }
    }

    Model {
        font_size: 32,
        font_color: (0., 121., 255.),
        background_color: (0., 0., 0.),
        sequence: vec![],
        _window_ids: window_ids,
    }
}

fn event(app: &App, model: &mut Model, event: WindowEvent) {
    match event {
        WindowEvent::KeyReleased(key) => match key {
            Key::K if app.keys.mods.ctrl() => model.font_size += 1,
            Key::J if app.keys.mods.ctrl() => model.font_size -= 1,
            Key::Q if &model.sequence[..] == &[Key::Q, Key::Q] => app.quit(),
            _ => {
                model.sequence.push(key);
            }
        },
        _ => {}
    };
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    draw.background().color(
        color::rgb::Srgba::new(
            model.background_color.0,
            model.background_color.1,
            model.background_color.2,
            1.,
        )
        .into_linear(),
    );

    let win_rect = app.main_window().rect();

    let (x, y) = (app.mouse.x, app.mouse.y);

    let coordinates = &format!("({}, {})", x, y);

    let coordinates = text(&coordinates)
        .font_size(model.font_size)
        .build(win_rect);

    draw.path()
        .fill()
        .color(
            color::rgb::Srgba::new(
                model.font_color.0,
                model.font_color.1,
                model.font_color.2,
                1.,
            )
            .into_linear(),
        )
        .x_y(x, y)
        .events(coordinates.path_events());

    // Write the result of our drawing to the window's frame.
    if let Err(error) = draw
        .to_frame(app, &frame)
        .map_err(|error| format!("{:?}", error))
    {
        log::error!("{}", error);
    }
}

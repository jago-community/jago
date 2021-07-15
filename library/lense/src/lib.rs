mod font;
mod lense;
mod parts;

use std::iter::Peekable;

use nannou::{
    app::App,
    color,
    event::{Key, Update, WindowEvent},
    frame::Frame,
    image,
    text::{text, Font},
    window,
};

author::error!(
    Incomplete,
    DrawError(String),
    parts::Error,
    font::Error,
    font_kit::error::SelectionError,
    font_kit::error::FontLoadingError,
);

pub fn handle<I: Iterator<Item = String>>(input: &mut Peekable<I>) -> Result<(), Error> {
    match input.peek() {
        Some(next) if next == "lense" => input.next(),
        _ => return Err(Error::Incomplete),
    };
    let viewer = nannou::app(model).view(view).update(update);
    Ok(viewer.run())
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

struct Model {
    font: Option<Font>,
    font_size: u32,
    font_color: (f32, f32, f32),
    background_color: (f32, f32, f32),
    sequence: Vec<Key>,
    logo: Option<nannou::wgpu::Texture>,
    initialize_error: Option<Error>,
    _window_ids: Vec<window::Id>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            font: None,
            font_size: 32,
            font_color: (0., 121., 255.),
            background_color: (0., 0., 0.),
            sequence: vec![],
            logo: None,
            initialize_error: None,
            _window_ids: vec![],
        }
    }
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
                parts::logo_icon(
                    image::Rgb::from([0, 121, 255]),
                    nannou::geom::Rect::from_w_h(16., 24.),
                )
                .ok(),
            );
        }
    }

    let maybe_logo = parts::logo(
        image::Rgb::from([0, 121, 255]),
        nannou::geom::Rect::from_w_h(100., 140.),
    )
    .map_err(Error::from);

    let logo = match maybe_logo {
        Ok(logo) => logo,
        Err(error) => {
            return Model {
                initialize_error: Some(error),
                ..Default::default()
            }
        }
    };

    let logo = nannou::wgpu::Texture::from_image(app, &logo);

    let maybe_font = font::get("AndaleMono").map_err(Error::from);

    let font = match maybe_font {
        Ok(font) => font,
        Err(error) => {
            return Model {
                initialize_error: Some(error),
                ..Default::default()
            }
        }
    };

    Model {
        font: Some(font),
        logo: Some(logo),
        _window_ids: window_ids,
        ..Default::default()
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

    draw.background()
        .color(color::rgb::Srgba::new(1., 1., 1., 1.).into_linear());

    let win_rect = app.main_window().rect();

    if let Some(error) = &model.initialize_error {
        let error = format!("{}", error);
        let error = text(&error).font_size(model.font_size).build(win_rect);
        draw.path()
            .fill()
            .color(color::BLUE)
            .events(error.path_events());
        return;
    }

    if let Some(logo) = &model.logo {
        draw.texture(&logo);
    }

    let (x, y) = (app.mouse.x, app.mouse.y);

    let mut jago = text("ō");

    if let Some(font) = &model.font {
        jago = jago.font(font.clone());
    }

    let jago = jago.font_size(32).build(win_rect);

    draw.path()
        .fill()
        .color(color::BLUE)
        .x_y(win_rect.left() + 8., win_rect.top() - 8.)
        .events(jago.path_events());

    let coordinates = &format!("({}, {})", x, y);

    let _coordinates = text(&coordinates)
        .font_size(model.font_size)
        .build(win_rect);

    //draw.path()
    //.fill()
    //.color(color::BLUE)
    //.x_y(x, y)
    //.events(coordinates.path_events());

    // Write the result of our drawing to the window's frame.
    if let Err(error) = draw
        .to_frame(app, &frame)
        .map_err(|error| format!("{:?}", error))
    {
        log::error!("{}", error);
    }
}

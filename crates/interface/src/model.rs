use nannou::{event::ModifiersState, prelude::*};

#[derive(Clone)]
pub enum Mode {
    Normal,
    Insert,
}

pub struct Model {
    pub cursor: Point2,
    pub scale: f32,
    pub factor: f32,
    pub logo: wgpu::Texture,
    pub mode: Mode,
    pub modifiers: ModifiersState,
    pub document: String,
}

pub fn handle(app: &App) -> Model {
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
        document: "greetings stranger".into(),
    }
}

use nannou::{
    color::rgb::Rgb,
    geom::Rect,
    winit::{self, window::Icon},
};

author::error!(winit::window::BadIcon);

pub fn logo(color: Rgb, bounds: Rect) -> Result<Icon, Error> {
    let pixels = vec![];
    let (width, height) = bounds.w_h();
    Icon::from_rgba(pixels, width as u32, height as u32).map_err(Error::from)
}

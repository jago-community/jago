use nannou::{
    geom::{Ellipse, Rect, Vec2},
    image::{self, DynamicImage, ImageBuffer},
    winit::{self, window::Icon},
};

author::error!(winit::window::BadIcon);

pub fn logo(color: image::Rgb<u8>, bounds: Rect) -> Result<DynamicImage, Error> {
    let (width, height) = bounds.w_h();

    let pixel_count = width as usize * height as usize;
    let mut pixels = ImageBuffer::new(width as u32, height as u32);

    let top = Rect::from_x_y_w_h(width / 2., 0., width * 0.75, height - width - 8.);

    let circle = Ellipse::new(
        Rect::from_x_y_w_h(width / 2., height * 0.3, width, width),
        80.,
    );

    let mut rng = rand::thread_rng();

    let sample = rand::seq::index::sample(&mut rng, pixel_count, pixel_count * 0.75 as usize);

    for index in sample {
        let x = index as f32 / width;
        let y = index as f32 / height;
        let xy = Vec2::new(x, y);

        if !top.contains(xy) || ellipse_contains(circle, xy) {
            continue;
        }

        pixels.put_pixel(x as u32, y as u32, color);
    }

    Ok(DynamicImage::ImageRgb8(pixels))
}

fn ellipse_contains(bounds: Ellipse, coordinates: Vec2) -> bool {
    let [x, y] = coordinates.to_array();
    let (h, k) = (bounds.rect.x.middle(), bounds.rect.y.middle());
    let (rx, ry) = (bounds.rect.right() - h, bounds.rect.top() - k);

    (x - h).sqrt() / rx.sqrt() + (y - k).sqrt() / (ry).sqrt() <= 1.
}

pub fn logo_icon(color: image::Rgb<u8>, bounds: Rect) -> Result<Icon, Error> {
    let image = logo(color, bounds)?;

    let pixels = image.into_rgba8().into_vec();

    Icon::from_rgba(pixels, bounds.w() as u32, bounds.h() as u32).map_err(Error::from)
}

/*
pub mod jago {
    use nannou::{
        geom::{Rect, Vec2},
        image::{self, DynamicImage, ImageBuffer},
    };

    use ::{
        nalgebra::{geometry::Point2, Unit},
        ncollide2d::{
            bounding_volume::AABB,
            math::{Isometry, Vector},
            shape::{Ball, FeatureId, Shape},
        },
    };

    #[derive(Clone)]
    struct Rectangle(Rect);

    impl Shape<f32> for Rectangle {
        fn aabb(&self, _: &Isometry<f32>) -> AABB<f32> {
            let x_min = self.0.left();
            let y_min = self.0.bottom();
            let x_max = self.0.right();
            let y_max = self.0.top();

            AABB::from_points([&Point2::new(x_min, y_min), &Point2::new(x_max, y_max)])
        }

        fn tangent_cone_contains_dir(
            &self,
            _feature: FeatureId,
            _m: &Isometry<f32>,
            _deformations: Option<&[f32]>,
            dir: &Unit<Vector<f32>>,
        ) -> bool {
            unimplemented!()
        }
    }

    pub fn logo(color: image::Rgb<u8>, bounds: Rect) -> Result<DynamicImage, super::Error> {
        let (width, height) = bounds.w_h();

        let pixel_count = width as usize * height as usize;
        let mut pixels = ImageBuffer::new(width as u32, height as u32);

        //let top = Cuboid::new(width / 2., 0., width * 0.75, height - width - 8.);

        let circle = ncollide2d::shape::Ball::new(width / 2.);

        let mut rng = rand::thread_rng();

        let sample = rand::seq::index::sample(&mut rng, pixel_count, pixel_count * 0.75 as usize);

        for index in sample {
            let x = index as f32 / width;
            let y = index as f32 / height;
            let xy = Vec2::new(x, y);
            if !top.contains(xy) || ellipse_contains(circle, xy) {
                continue;
            }

            pixels.put_pixel(x as u32, y as u32, color);
        }

        Ok(DynamicImage::ImageRgb8(pixels))
    }
}
*/

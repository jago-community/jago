use ::{
    nalgebra::{
        base::{Scalar, Vector2},
        geometry::Point2,
        SimdPartialOrd,
    },
    num_traits::identities::Zero,
    std::cmp::{max, min},
};

struct Rectangle {
    xs: Vector2<f32>,
    ys: Vector2<f32>,
}

impl Rectangle {
    fn from_points(a: Point2<f32>, b: Point2<f32>) -> Self {
        Self {
            xs: Vector2::new(a.x, b.x),
            ys: Vector2::new(a.y, b.y),
        }
    }

    fn x_upper(&self) -> f32 {
        self.xs.max()
    }

    fn x_lower(&self) -> f32 {
        self.xs.min()
    }

    fn x_center(&self) -> f32 {
        (self.xs.x - self.xs.y).abs() / 2.
    }

    fn y_upper(&self) -> f32 {
        self.ys.max()
    }

    fn y_lower(&self) -> f32 {
        self.ys.min()
    }

    fn y_center(&self) -> f32 {
        (self.ys.x - self.ys.y).abs() / 2.
    }
}

struct Logo {
    bounds: Rectangle,
}

impl Logo {
    fn new(bounds: Rectangle) -> Self {
        Self { bounds }
    }

    fn bar(&self) -> Rectangle {
        let x_lower = self.bounds.x_lower();

        let rectangle = Rectangle::from_points(
            Point2::new(self.bounds.x_lower() + 24., self.bounds.y_upper()),
            Point2::new(self.bounds.x_lower() + 24., self.bounds.x_upper() - 24.),
        );

        rectangle
    }
}

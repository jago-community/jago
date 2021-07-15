author::error!(Incomplete);

use {
    nalgebra::base::SMatrix,
    parry2d::{
        bounding_volume::AABB,
        math::{Point, Real},
    },
};

fn logo<'a, const R: usize, const C: usize>(
    matrix: &'a SMatrix<Real, R, C>,
) -> Result<&'a SMatrix<Real, R, C>, Error> {
    let w = R as f32;
    let h = C as f32;

    let top_bar = AABB::new(Point::new(w * 0.20, 0.), Point::new(w * 0.8, h * 0.1));

    unimplemented!()
}

#[test]
fn test_concept() {
    use {
        nalgebra::{Isometry2, Vector2},
        parry2d::{
            bounding_volume::AABB,
            math::{Point, Real},
            query,
            shape::{Ball, Cuboid},
        },
    };

    let cuboid = Cuboid::new(Vector2::new(1.0, 1.0));
    let ball = Ball::new(1.0);
    let prediction = 1.0;

    let cuboid_pos = Isometry2::identity();
    let ball_pos_penetrating = Isometry2::translation(1.0, 1.0);
    let ball_pos_in_prediction = Isometry2::translation(2.0, 2.0);
    let ball_pos_too_far = Isometry2::translation(3.0, 3.0);

    let ctct_penetrating = query::contact(
        &ball_pos_penetrating,
        &ball,
        &cuboid_pos,
        &cuboid,
        prediction,
    )
    .unwrap();
    let ctct_in_prediction = query::contact(
        &ball_pos_in_prediction,
        &ball,
        &cuboid_pos,
        &cuboid,
        prediction,
    )
    .unwrap();
    let ctct_too_far =
        query::contact(&ball_pos_too_far, &ball, &cuboid_pos, &cuboid, prediction).unwrap();

    assert!(ctct_penetrating.unwrap().dist <= 0.0);
    assert!(ctct_in_prediction.unwrap().dist >= 0.0);
    assert_eq!(ctct_too_far, None);
}

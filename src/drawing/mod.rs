use bevy::prelude::*;

pub trait Drawable {
    fn draw(&self, base_pos: Vec2, gz: &mut Gizmos);
}

pub fn draw_polygon(base_pos: Vec2, points: &[Vec2], color: Color, gz: &mut Gizmos) {
    for ix in 0..points.len() {
        gz.line_2d(
            base_pos + points[ix],
            base_pos + points[(ix + 1) % points.len()],
            color,
        );
    }
}

#[macro_export]
macro_rules! drawable {
    ($type: ty, $fname: ident) => {
        fn $fname(mut gz: Gizmos, things: Query<(&$type, &Transform)>) {
            for (thing, transform) in things.iter() {
                thing.draw(
                    Vec2::new(transform.translation.x, transform.translation.y),
                    &mut gz,
                );
            }
        }
    };
}

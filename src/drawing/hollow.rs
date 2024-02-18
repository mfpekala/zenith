use bevy::prelude::*;

use crate::math::regular_polygon;

pub trait HollowDrawable {
    fn draw_hollow(&self, base_pos: Vec2, gz: &mut Gizmos);
}

pub fn draw_hollow_polygon(base_pos: Vec2, points: &[Vec2], color: Color, gz: &mut Gizmos) {
    for ix in 0..points.len() {
        gz.line_2d(
            base_pos + points[ix],
            base_pos + points[(ix + 1) % points.len()],
            color,
        );
    }
}

#[macro_export]
macro_rules! hollow_drawable {
    ($type: ty, $fname: ident) => {
        fn $fname(mut gz: Gizmos, things: Query<(&$type, &GlobalTransform)>) {
            for (thing, transform) in things.iter() {
                thing.draw_hollow(
                    Vec2::new(transform.translation().x, transform.translation().y),
                    &mut gz,
                );
            }
        }
    };
}

#[derive(Component)]
pub struct CircleMarker {
    pub radius: f32,
    pub color: Color,
    pub shown: bool,
}
impl CircleMarker {
    pub fn new(radius: f32, color: Color) -> Self {
        Self {
            radius,
            color,
            shown: true,
        }
    }
}

#[derive(Component)]
pub struct EquiTriangleMarker {
    pub radius: f32,
    pub color: Color,
    pub shown: bool,
}
impl EquiTriangleMarker {
    pub fn new(radius: f32, color: Color) -> Self {
        Self {
            radius,
            color,
            shown: true,
        }
    }
}

fn draw_hollow_circles(
    circles_n_transforms: Query<(&CircleMarker, &GlobalTransform)>,
    mut gz: Gizmos,
) {
    for (circle, tran) in circles_n_transforms.iter() {
        if !circle.shown {
            continue;
        }
        gz.circle_2d(tran.translation().truncate(), circle.radius, circle.color);
    }
}

fn draw_hollow_equi_triangles(
    tris_n_transforms: Query<(&EquiTriangleMarker, &Transform)>,
    mut gz: Gizmos,
) {
    for (tri, tran) in tris_n_transforms.iter() {
        if !tri.shown {
            continue;
        }
        let center = tran.translation.truncate();
        let points = regular_polygon(3, 0.0, tri.radius);
        draw_hollow_polygon(center, &points, tri.color, &mut gz);
    }
}

#[derive(Component)]
pub struct ShrinkingCircle {
    pub max_size: f32,
    pub size: f32,
    pub speed: f32,
}

#[derive(Bundle)]
pub struct ShrinkingCircleBundle {
    pub shrink: ShrinkingCircle,
    pub inner_marker: CircleMarker,
}
impl ShrinkingCircleBundle {
    pub fn new(size: f32, speed: f32) -> Self {
        Self {
            shrink: ShrinkingCircle {
                max_size: size,
                size,
                speed,
            },
            inner_marker: CircleMarker::new(size, Color::TOMATO),
        }
    }
}

fn update_shrinking_circles(mut shrinks: Query<(&mut ShrinkingCircle, &mut CircleMarker)>) {
    for (mut shrink, mut cm) in shrinks.iter_mut() {
        cm.radius = shrink.size;
        shrink.size -= shrink.speed;
        if shrink.size <= 0.0 {
            shrink.size += shrink.max_size
        }
    }
}

pub fn register_hollow_drawing(app: &mut App) {
    app.add_systems(
        Update,
        (
            draw_hollow_circles,
            draw_hollow_equi_triangles,
            update_shrinking_circles,
        ),
    );
}

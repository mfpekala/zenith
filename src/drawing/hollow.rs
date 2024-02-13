use bevy::prelude::*;

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

fn draw_hollow_circles(circles_n_transforms: Query<(&CircleMarker, &Transform)>, mut gz: Gizmos) {
    for (circle, tran) in circles_n_transforms.iter() {
        if !circle.shown {
            continue;
        }
        gz.circle_2d(tran.translation.truncate(), circle.radius, circle.color);
    }
}

pub fn register_hollow_drawing(app: &mut App) {
    app.add_systems(Update, draw_hollow_circles);
}

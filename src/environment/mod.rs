use bevy::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct Comet {
    pub points: Vec<Vec2>,
    pub has_gravity: bool,
    pub gravity_strength: f32,
    pub gravity_reach: f32,
}
impl Comet {
    pub fn render(&self, base_pos: Vec2, gz: &mut Gizmos) {
        for ix in 0..self.points.len() {
            gz.line_2d(
                base_pos + self.points[ix],
                base_pos + self.points[(ix + 1) % self.points.len()],
                Color::WHITE,
            );
        }
    }
}

#[derive(Bundle)]
pub struct CometBundle {
    comet: Comet,
    spatial: SpatialBundle,
}
impl CometBundle {
    pub fn new(base_pos: Vec2, comet: Comet) -> Self {
        CometBundle {
            comet,
            spatial: SpatialBundle {
                transform: Transform {
                    translation: base_pos.extend(0.0),
                    ..default()
                },
                ..default()
            },
        }
    }
}

fn render_comets(mut gz: Gizmos, comets: Query<(&Comet, &Transform)>) {
    for (comet, transform) in comets.iter() {
        comet.render(
            Vec2::new(transform.translation.x, transform.translation.y),
            &mut gz,
        );
    }
}

fn test_comets(mut commands: Commands) {
    let bundle = CometBundle::new(
        Vec2::new(0.0, 0.0),
        Comet {
            points: vec![
                Vec2::new(100.0, 100.0),
                Vec2::new(-200.0, 10.0),
                Vec2::new(-100.0, -300.0),
                Vec2::new(100.0, -100.0),
            ],
            has_gravity: false,
            gravity_strength: 0.0,
            gravity_reach: 0.0,
        },
    );
    commands.spawn(bundle);
}

pub fn register_environment(app: &mut App) {
    app.add_systems(Startup, test_comets);
    app.add_systems(Update, render_comets);
}

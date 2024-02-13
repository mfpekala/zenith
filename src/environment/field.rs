use super::rock::Rock;
use crate::{
    drawing::hollow::{draw_hollow_polygon, CircleMarker, HollowDrawable},
    hollow_drawable,
    math::{get_shell, MathLine},
    meta::game_state::{in_editor, in_level},
};
use bevy::prelude::*;

/// NOTE: Points MUST be in clockwise order
#[derive(Component, Clone, Debug)]
pub struct Field {
    pub points: Vec<Vec2>,
    pub particle_manager: ForceParticleManager,
    pub strength: f32,
    pub dir: Vec2,
    pub drag: f32,
}
impl HollowDrawable for Field {
    fn draw_hollow(&self, base_pos: Vec2, gz: &mut Gizmos) {
        draw_hollow_polygon(base_pos, &self.points, Color::YELLOW, gz);
    }
}
impl Field {
    pub fn effective_mult(&self, pos: &Vec2, base_pos: &Vec2, radius: f32) -> f32 {
        let lines = MathLine::from_points(&self.points);
        let mut max_dist = f32::MIN;
        let adjusted_pos = *pos - *base_pos;
        for line in lines {
            let signed_dist = line.signed_distance_from_point(&adjusted_pos);
            max_dist = max_dist.max(signed_dist);
        }
        max_dist = (-1.0 * (max_dist / radius) + 1.0) / 2.0;
        max_dist.min(1.0).max(0.0)
    }

    pub fn uniform_around_rock(rock: &Rock, reach: f32, strength: f32) -> Vec<Self> {
        let shell = get_shell(&rock.points, reach);
        let mut regions: Vec<Field> = vec![];
        for ix in 0..rock.points.len() {
            let p1 = shell[ix];
            let p2 = shell[(ix + 1) % shell.len()];
            let diff = (p2 - p1).normalize();
            let points = vec![
                p1,
                p2,
                rock.points[(ix + 1) % rock.points.len()],
                rock.points[ix],
            ];
            let region = Field {
                particle_manager: ForceParticleManager::new(&points),
                points,
                strength,
                dir: Vec2 {
                    x: diff.y,
                    y: -diff.x,
                },
                drag: 0.0003,
            };
            regions.push(region);
        }
        regions
    }
}
hollow_drawable!(Field, draw_fields);

#[derive(Bundle)]
pub struct FieldBundle {
    pub field: Field,
    pub spatial: SpatialBundle,
}
impl FieldBundle {
    pub fn new(field: Field) -> Self {
        Self {
            field,
            spatial: default(),
        }
    }
}

#[derive(Component)]
pub struct ForceParticle {
    field_id: Entity,
}
impl ForceParticle {
    pub fn new(field_id: Entity) -> Self {
        Self { field_id }
    }
}

#[derive(Bundle)]
struct ForceParticleBundle {
    particle: ForceParticle,
    spatial: SpatialBundle,
    circle: CircleMarker,
}
impl ForceParticleBundle {
    pub fn new(particle: ForceParticle, pos: Vec2) -> Self {
        Self {
            particle,
            spatial: SpatialBundle::from_transform(Transform::from_translation(pos.extend(0.0))),
            circle: CircleMarker::new(
                1.0,
                Color::Rgba {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 0.0,
                },
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ForceParticleManager {
    pub ticks: u32,
    pub countup: u32,
    pub aabb: [Vec2; 2],
}
impl ForceParticleManager {
    pub fn new(points: &Vec<Vec2>) -> Self {
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        for point in points {
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }
        Self {
            ticks: u32::MAX,
            countup: 50,
            aabb: [Vec2 { x: min_x, y: min_y }, Vec2 { x: max_x, y: max_y }],
        }
    }

    pub fn increment(&mut self) -> bool {
        if self.ticks >= self.countup {
            self.ticks = 0;
            true
        } else {
            self.ticks += 1;
            false
        }
    }

    pub fn spawn_particles(&self, commands: &mut Commands, pos: Vec2, field_id: Entity) {
        let density: f32 = 15.0;
        let mut points: Vec<Vec2> = vec![];
        let mut x = self.aabb[0].x;
        while x < self.aabb[1].x {
            points.push(
                pos + Vec2 {
                    x,
                    y: self.aabb[0].y,
                },
            );
            points.push(
                pos + Vec2 {
                    x,
                    y: self.aabb[1].y,
                },
            );
            x += density;
        }
        let mut y = self.aabb[0].y;
        while y < self.aabb[1].y {
            points.push(
                pos + Vec2 {
                    x: self.aabb[0].x,
                    y,
                },
            );
            points.push(
                pos + Vec2 {
                    x: self.aabb[1].x,
                    y,
                },
            );
            y += density;
        }
        for point in points {
            commands.spawn(ForceParticleBundle::new(
                ForceParticle::new(field_id),
                point,
            ));
        }
    }
}

fn update_force_particles(
    mut commands: Commands,
    mut fps: Query<(Entity, &mut Transform, &ForceParticle, &mut CircleMarker)>,
    fields: Query<(&Field, &GlobalTransform)>,
) {
    for (id, mut tran, fp, mut cm) in fps.iter_mut() {
        let Ok((parent_field, parent_tran)) = fields.get(fp.field_id) else {
            // Despawn if parent not found
            commands.entity(id).despawn();
            continue;
        };
        let mut live_box = parent_field.particle_manager.aabb;
        live_box[0] += parent_tran.translation().truncate();
        live_box[1] += parent_tran.translation().truncate();
        if tran.translation.x < live_box[0].x
            || tran.translation.x > live_box[1].x
            || tran.translation.y < live_box[0].y
            || tran.translation.y > live_box[1].y
        {
            // Despawn if out of bounding box
            commands.entity(id).despawn();
            continue;
        }
        // We're still alive! Move
        tran.translation += (parent_field.dir * parent_field.strength * 5.0).extend(0.0);
        // Only show us if we're in the box
        if parent_field.effective_mult(
            &tran.translation.truncate(),
            &parent_tran.translation().truncate(),
            1.0,
        ) > 0.0
        {
            cm.color = Color::Rgba {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: 0.5,
            };
        } else {
            cm.color = Color::Rgba {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: 0.0,
            }
        }
    }
}

fn manage_force_particles(
    mut commands: Commands,
    mut fields: Query<(Entity, &mut Field, &GlobalTransform)>,
) {
    for (id, mut field, tran) in fields.iter_mut() {
        if field.particle_manager.increment() {
            field.particle_manager.spawn_particles(
                &mut commands,
                tran.translation().truncate(),
                id,
            );
        }
    }
}

pub fn register_fields(app: &mut App) {
    app.add_systems(
        Update,
        (draw_fields, update_force_particles, manage_force_particles)
            .run_if(in_editor.or_else(in_level)),
    );
}

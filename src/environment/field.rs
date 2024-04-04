use super::rock::Rock;
use crate::{
    drawing::layering::sprite_layer, math::get_shell, physics::collider::ColliderTriggerBundle,
};
use bevy::{prelude::*, render::view::RenderLayers};

/// NOTE: Points MUST be in clockwise order
#[derive(Component, Clone, Debug)]
pub struct Field {
    pub points: Vec<Vec2>,
    pub strength: f32,
    pub dir: Vec2,
    pub drag: f32,
}
impl Field {
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

#[derive(Bundle)]
pub struct FieldBundle {
    pub field: Field,
    pub spatial: SpatialBundle,
    pub render_layers: RenderLayers,
}
impl FieldBundle {
    pub fn spawn(commands: &mut Commands, base_pos: Vec2, field: Field) {
        commands
            .spawn(Self {
                field: field.clone(),
                spatial: SpatialBundle::from_transform(Transform::from_translation(
                    base_pos.extend(0.0),
                )),
                render_layers: sprite_layer(),
            })
            .with_children(|parent| {
                let points = field
                    .points
                    .iter()
                    .map(|p| {
                        let r = (base_pos + *p).round();
                        IVec2 {
                            x: r.x as i32,
                            y: r.y as i32,
                        }
                    })
                    .collect();
                let cs = ColliderTriggerBundle::new(points, true);
                parent.spawn(cs);
            });
    }
}

pub fn register_fields(app: &mut App) {}

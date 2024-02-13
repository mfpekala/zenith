use bevy::prelude::*;

use super::{
    field::{Field, FieldBundle},
    rock::{Rock, RockBundle},
};

#[derive(Component)]
pub struct Planet;

#[derive(Bundle)]
pub struct PlanetBundle {
    planet: Planet,
    rock: RockBundle,
}
impl PlanetBundle {
    pub fn new(
        base_pos: Vec2,
        rock: Rock,
        reach_n_strength: Option<(f32, f32)>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> (Self, Vec<Field>) {
        let mut rock_bundle = RockBundle::from_rock(rock.clone(), meshes);
        rock_bundle.mesh.transform.translation = base_pos.extend(0.0);
        match reach_n_strength {
            Some((reach, strength)) => (
                Self {
                    planet: Planet,
                    rock: rock_bundle,
                },
                Field::uniform_around_rock(&rock, reach, strength),
            ),
            None => (
                Self {
                    planet: Planet,
                    rock: rock_bundle,
                },
                vec![],
            ),
        }
    }

    pub fn spawn(
        commands: &mut Commands,
        base_pos: Vec2,
        rock: Rock,
        reach_n_strength: Option<(f32, f32)>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) {
        let (pb, fields) = PlanetBundle::new(base_pos, rock, reach_n_strength, meshes);
        commands.spawn(pb).with_children(|comms| {
            for field in fields {
                comms.spawn(FieldBundle::new(field));
            }
        });
    }
}

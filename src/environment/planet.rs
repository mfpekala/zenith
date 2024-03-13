//! Nothing more than a helpful idea of a rock tied to some sanely-defined fields

use super::{
    field::{Field, FieldBundle},
    rock::{Rock, RockBundle, RockResources},
};
use bevy::prelude::*;

pub fn spawn_planet(
    commands: &mut Commands,
    base_pos: Vec2,
    rock: Rock,
    reach: f32,
    strength: f32,
    meshes: &mut ResMut<Assets<Mesh>>,
    rock_res: &Res<RockResources>,
) {
    let fields = Field::uniform_around_rock(&rock, reach, strength);
    RockBundle::spawn(commands, base_pos, rock, meshes, rock_res);
    for field in fields {
        FieldBundle::spawn(commands, base_pos, field);
    }
}

pub mod background;
pub mod field;
pub mod goal;
pub mod particle;
pub mod rock;
pub mod starting_point;

use self::{goal::register_goals, particle::register_particles};
use bevy::prelude::*;

pub fn register_environment(app: &mut App) {
    register_goals(app);
    register_particles(app);
}

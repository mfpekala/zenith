pub mod background;
pub mod field;
pub mod goal;
pub mod particle;
pub mod replenish;
pub mod rock;
pub mod segment;
pub mod start;

use self::{particle::register_particles, replenish::update_replenishes};
use bevy::prelude::*;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        register_particles(app);
        app.add_systems(Update, update_replenishes);
    }
}

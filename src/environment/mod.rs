pub mod background;
pub mod field;
pub mod goal;
pub mod live_poly;
pub mod particle;
pub mod replenish;
pub mod rock;
pub mod segment;
pub mod start;

use self::{particle::register_particles, replenish::update_replenishes};
use bevy::prelude::*;
use live_poly::mark_live_polys_ready;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        register_particles(app);
        app.add_systems(Update, update_replenishes);
        app.add_systems(FixedUpdate, mark_live_polys_ready);
    }
}

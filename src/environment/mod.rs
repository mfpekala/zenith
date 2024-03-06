pub mod background;
pub mod field;
pub mod goal;
pub mod particle;
pub mod planet;
pub mod rock;
pub mod starting_point;

use self::{field::register_fields, goal::register_goals, rock::register_rocks};
use bevy::prelude::*;

pub fn register_environment(app: &mut App) {
    register_fields(app);
    register_goals(app);
    register_rocks(app);
}

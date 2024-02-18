pub mod field;
pub mod goal;
pub mod planet;
pub mod rock;

use self::{field::register_fields, rock::register_rocks};
use bevy::prelude::*;

pub fn register_environment(app: &mut App) {
    register_rocks(app);
    register_fields(app);
}

use self::hollow::register_hollow_drawing;
use bevy::prelude::*;

pub mod hollow;
pub mod mesh;

pub fn register_drawing(app: &mut App) {
    register_hollow_drawing(app);
}

use bevy::prelude::*;

use self::music::MusicPlugin;

pub mod music;

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MusicPlugin);
    }
}

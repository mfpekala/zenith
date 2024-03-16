use bevy::prelude::*;

#[derive(Component)]
pub struct EyeOfTheStormMusic;

fn setup_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioBundle {
            source: asset_server.load("music/Eye of the Storm.ogg"),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                paused: true,
                // paused: false,
                ..default()
            },
        },
        EyeOfTheStormMusic,
    ));
}

pub(super) struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_music);
    }
}

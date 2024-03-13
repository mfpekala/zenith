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

pub fn register_music(app: &mut App) {
    app.add_systems(Startup, setup_music);
}

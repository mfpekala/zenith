use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};
use serde::{Deserialize, Serialize};

use crate::math::Spleen;

use super::SoundSettings;

#[derive(Component)]
struct MusicMarker;

#[derive(Component)]
struct MusicMarkerChild;

#[derive(Clone, Copy, Debug, Reflect, Serialize, Deserialize, PartialEq)]
pub enum MusicKind {
    EyeOfTheStorm,
    APlaceICallHome,
    SandCastles,
}
impl MusicKind {
    fn to_asset_path(&self) -> String {
        match self {
            Self::EyeOfTheStorm => "music/eye_of_the_storm.ogg".into(),
            Self::APlaceICallHome => "music/a_place_i_call_home.ogg".into(),
            Self::SandCastles => "music/sand_castles.ogg".into(),
        }
    }

    /// How loud should it be regularly to balance with the other music + sounds
    fn to_volume_adjustment(&self) -> f32 {
        match self {
            Self::EyeOfTheStorm => 0.6,
            Self::APlaceICallHome => 0.1,
            Self::SandCastles => 0.2,
        }
    }
}

#[derive(Clone, Debug, Reflect)]
struct MusicTransition {
    to: Option<MusicKind>,
    fade_out: Option<Timer>,
    fade_in: Option<Timer>,
}

#[derive(Resource, Debug, Default, Clone, Reflect)]
pub struct MusicManager {
    current: Option<MusicKind>,
    transition: Option<MusicTransition>,
}
impl MusicManager {
    const FADE_TIME: f32 = 0.3;

    pub fn fade_to_song(&mut self, song: Option<MusicKind>) {
        if song == self.current && self.transition.is_none() {
            // Don't need to do anything
            return;
        }
        self.transition = Some(MusicTransition {
            to: song,
            fade_out: Some(Timer::from_seconds(Self::FADE_TIME, TimerMode::Once)),
            fade_in: None,
        })
    }
}

fn setup_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((MusicMarker, Name::new("music")))
        .with_children(|parent| {
            parent.spawn((
                AudioBundle {
                    source: asset_server.load("music/a_place_i_call_home.ogg"),
                    settings: PlaybackSettings {
                        mode: bevy::audio::PlaybackMode::Loop,
                        volume: Volume::new(0.0),
                        ..default()
                    },
                },
                MusicMarkerChild,
                Name::new("music_child"),
            ));
        });
}

fn update_music(
    music_parent: Query<Entity, With<MusicMarker>>,
    mut music_child: Query<(&AudioSink, &mut PlaybackSettings), With<MusicMarkerChild>>,
    sound_settings: Res<SoundSettings>,
    mut manager: ResMut<MusicManager>,
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
) {
    let parent_eid = music_parent.single();
    let child = music_child.get_single_mut();
    let kind_factor = manager
        .current
        .as_ref()
        .map(|kind| kind.to_volume_adjustment())
        .unwrap_or(0.0);
    let settings_factor = sound_settings.main_volume * sound_settings.music_volume;
    let set_volume = |x: f32| {
        if let Ok((audio_sink, mut playback_settings)) = child {
            audio_sink.set_volume(kind_factor * settings_factor * x);
            playback_settings.volume = Volume::new(kind_factor * settings_factor * x);
        }
    };
    let mut go_to = None;
    let mut stop_transition = false;
    match manager.transition.as_mut() {
        Some(transition) => {
            let mut go_next = false;
            if let Some(fade_out) = transition.fade_out.as_mut() {
                fade_out.tick(time.delta());
                let x = Spleen::EaseInOutCubic.bound_interp(fade_out.fraction(), 1.0, 0.0);
                set_volume(x);
                go_next = fade_out.finished();
            } else if let Some(fade_in) = transition.fade_in.as_mut() {
                fade_in.tick(time.delta());
                let x = Spleen::EaseInOutCubic.bound_interp(fade_in.fraction(), 0.0, 1.0);
                set_volume(x);
                go_next = fade_in.finished();
            }
            if go_next {
                if transition.fade_out.is_some() {
                    transition.fade_out = None;
                    transition.fade_in = Some(Timer::from_seconds(
                        MusicManager::FADE_TIME,
                        TimerMode::Once,
                    ));
                    go_to = transition.to;
                } else if transition.fade_in.is_some() {
                    transition.fade_out = None;
                    transition.fade_in = None;
                    stop_transition = true;
                }
            }
        }
        None => {
            set_volume(1.0);
        }
    }
    if let Some(kind) = go_to {
        commands.entity(parent_eid).despawn_descendants();
        commands.entity(parent_eid).with_children(|parent| {
            parent.spawn((
                AudioBundle {
                    source: asset_server.load(kind.to_asset_path()),
                    settings: PlaybackSettings {
                        mode: PlaybackMode::Loop,
                        volume: Volume::new(0.0),
                        paused: false,
                        ..default()
                    },
                },
                MusicMarkerChild,
                Name::new("music_child"),
            ));
        });
        manager.current = Some(kind);
    }
    if stop_transition {
        manager.transition = None;
    }
}

pub(super) struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MusicManager::default());
        app.add_systems(Startup, setup_music);
        app.add_systems(Update, update_music);
    }
}

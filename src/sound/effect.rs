use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};

use crate::{
    camera::CameraMarker,
    math::Spleen,
    meta::consts::{SCREEN_HEIGHT, SCREEN_WIDTH},
    physics::dyno::IntMoveable,
};

use super::SoundSettings;

/// Off screen sounds will decay from base_volume to off_volume using a spleen
/// Decay rate controls how fast this happens
#[derive(Debug, Clone)]
pub struct SoundOffscreenOptions {
    off_volume: f32,
    decay_rate: f32,
}
impl Default for SoundOffscreenOptions {
    fn default() -> Self {
        Self {
            off_volume: 0.0,
            decay_rate: 1.0,
        }
    }
}

#[derive(Component, Debug)]
pub struct SoundEffect {
    path: String,
    base_volume: f32,
    offscreen: Option<SoundOffscreenOptions>,
    repeat: bool,
}
impl Default for SoundEffect {
    fn default() -> Self {
        Self {
            path: "".into(),
            base_volume: 1.0,
            offscreen: Some(SoundOffscreenOptions::default()),
            repeat: false,
        }
    }
}
impl SoundEffect {
    pub fn universal(path: &str, base_volume: f32, repeat: bool) -> SoundEffect {
        Self {
            path: path.into(),
            base_volume,
            offscreen: None,
            repeat,
        }
    }

    pub fn spatial(path: &str, base_volume: f32, repeat: bool) -> SoundEffect {
        Self {
            path: path.into(),
            base_volume,
            offscreen: Some(SoundOffscreenOptions::default()),
            repeat,
        }
    }
}

fn spawn_sound_effects(
    mut commands: Commands,
    lacking: Query<(Entity, &SoundEffect), Without<PlaybackSettings>>,
    asset_server: Res<AssetServer>,
    sound_settings: Res<SoundSettings>,
) {
    for (eid, effect) in lacking.iter() {
        commands.entity(eid).insert(AudioBundle {
            source: asset_server.load(&effect.path),
            settings: PlaybackSettings {
                mode: if effect.repeat {
                    PlaybackMode::Loop
                } else {
                    PlaybackMode::Despawn
                },
                volume: Volume::new(sound_settings.effect_volume * effect.base_volume),
                ..default()
            },
        });
    }
}

fn update_sound_effect_volume(
    mut sounds: Query<(
        &SoundEffect,
        &mut PlaybackSettings,
        &mut AudioSink,
        Option<&GlobalTransform>,
    )>,
    cam_q: Query<&IntMoveable, With<CameraMarker>>,
    sound_settings: Res<SoundSettings>,
) {
    let Ok(cam_mv) = cam_q.get_single() else {
        return;
    };
    for (effect, mut playback, sink, gtran) in sounds.iter_mut() {
        let mult = match (effect.offscreen.clone(), gtran) {
            (Some(offscreen), Some(gtran)) => {
                let x_dist = (gtran.translation().x as i32)
                    .abs_diff(cam_mv.pos.x)
                    .saturating_sub(SCREEN_WIDTH as u32 / 2);
                let y_dist = (gtran.translation().y as i32)
                    .abs_diff(cam_mv.pos.y)
                    .saturating_sub(SCREEN_HEIGHT as u32 / 2);
                let clamp_max = (SCREEN_HEIGHT as f32) / offscreen.decay_rate;
                let off_dist = (x_dist.max(y_dist) as f32).clamp(0.0, clamp_max) / clamp_max;
                let mult = Spleen::EaseOutCubic.bound_interp(off_dist, 1.0, offscreen.off_volume);
                mult
            }
            _ => 1.0,
        };
        playback.volume = Volume::new(
            sound_settings.main_volume * sound_settings.effect_volume * effect.base_volume * mult,
        );
        sink.set_volume(playback.volume.abs());
    }
}

pub(super) struct SoundEffectPlugin;

impl Plugin for SoundEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_sound_effects, update_sound_effect_volume));
    }
}

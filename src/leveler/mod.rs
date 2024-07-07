use bevy::prelude::*;
use load::{actively_load, destroy_level, did_level_change, is_actively_loading_level, start_load};

use crate::{
    camera::CameraMarker,
    drawing::effects::{ScreenEffect, ScreenEffectManager},
    meta::{
        consts::{FRAMERATE, MENU_GROWTH, MENU_HEIGHT, MENU_WIDTH},
        game_state::{in_level, left_level, GameState, LevelState, MenuState, MetaState},
        progress::{ActiveSaveFile, GameProgress},
    },
    physics::dyno::{IntDyno, IntMoveable},
    ship::{Dead, Ship},
    sound::effect::SoundEffect,
};

pub mod load;

fn progress_level(
    mut ships: Query<(&mut Ship, &IntDyno), Without<Dead>>,
    gs: Res<GameState>,
    mut game_progress: Query<&mut GameProgress, With<ActiveSaveFile>>,
    mut screen_effect: ResMut<ScreenEffectManager>,
    cam: Query<&IntMoveable, With<CameraMarker>>,
    mut commands: Commands,
) {
    let Some(level_state) = gs.get_level_state() else {
        // warn!("Weird stuff happening in progress_level level_state");
        return;
    };
    let Ok(mut game_progress) = game_progress.get_single_mut() else {
        // warn!("Weird stuff happening in progress_level game_progress");
        return;
    };
    let Ok(cam) = cam.get_single() else {
        return;
    };
    let saturated_goal = ships.iter().any(|(ship, dyno)| {
        // fuck it hopefully the compiler optimizes this
        let enough_time = !ship.finished && ship.time_in_goal > FRAMERATE as f32 * 0.75;
        let close_enough = ship.dist_to_goal_center_sq < 1.0;
        let slow_enough = dyno.vel.length_squared() < 0.05;
        enough_time && slow_enough && close_enough
    });
    if !saturated_goal {
        return;
    }
    commands.spawn(SoundEffect::universal(
        "sound_effects/level_transport.ogg",
        0.4,
        false,
    ));
    for (mut ship, _) in ships.iter_mut() {
        ship.finished = true;
    }
    let Some((_, ship_dyno)) = ships.iter().next() else {
        return;
    };
    let mut go_to_meta = |meta: MetaState, include_unfade: bool| {
        let mut pos = (ship_dyno.ipos.truncate() - cam.pos.truncate()) * MENU_GROWTH as i32;
        if pos.x.abs() > MENU_WIDTH as i32 / 2 || pos.y.abs() > MENU_HEIGHT as i32 / 2 {
            pos = IVec2::ZERO;
        }
        screen_effect.queue_effect(ScreenEffect::CircleIn {
            from_pos: pos,
            to_state: Some(GameState { meta, pause: None }),
        });
        screen_effect.queue_effect(ScreenEffect::Black);
        if include_unfade {
            screen_effect.queue_effect(ScreenEffect::UnfadeToBlack);
        }
    };
    match game_progress.try_mark_completed(level_state.kind, level_state.id.clone()) {
        Err(e) => {
            warn!("Can't mark completed: {e:?}");
            go_to_meta(MetaState::Menu(MenuState::GalaxyOverworld), true);
        }
        Ok(_) => {
            let galaxy_progress = game_progress.get_galaxy_progress(level_state.kind);
            match galaxy_progress.next_level {
                Some(level_id) => {
                    go_to_meta(
                        MetaState::Level(LevelState::from_galaxy_n_level(
                            level_state.kind,
                            level_id,
                        )),
                        false,
                    );
                }
                None => {
                    // TODO: This implies the galaxy was just completed. It should flag a fun effect on the galaxy overworld
                    go_to_meta(MetaState::Menu(MenuState::GalaxyOverworld), true);
                }
            }
        }
    }
}

pub struct LevelerPlugin;
impl Plugin for LevelerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, start_load.run_if(did_level_change));
        app.add_systems(Update, actively_load.run_if(is_actively_loading_level));
        app.add_systems(Update, destroy_level.run_if(left_level));
        app.add_systems(Update, progress_level.run_if(in_level));
    }
}

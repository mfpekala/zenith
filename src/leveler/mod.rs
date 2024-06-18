use bevy::prelude::*;
use load::{actively_load, destroy_level, did_level_change, is_actively_loading_level, start_load};

use crate::{
    drawing::effects::{ScreenEffect, ScreenEffectManager},
    meta::{
        consts::FRAMERATE,
        game_state::{in_level, left_level, GameState, LevelState, MenuState, MetaState},
        progress::{ActiveSaveFile, GameProgress},
    },
    ship::Ship,
};

pub mod load;

fn progress_level(
    mut ships: Query<&mut Ship>,
    gs: Res<GameState>,
    mut game_progress: Query<&mut GameProgress, With<ActiveSaveFile>>,
    mut screen_effect: ResMut<ScreenEffectManager>,
) {
    let Some(level_state) = gs.get_level_state() else {
        warn!("Weird stuff happening in progress_level");
        return;
    };
    let Ok(mut game_progress) = game_progress.get_single_mut() else {
        warn!("Weird stuff happening in progress_level");
        return;
    };
    let saturated_goal = ships
        .iter()
        .any(|ship| !ship.finished && ship.time_in_goal > FRAMERATE as f32 * 0.75);
    if !saturated_goal {
        return;
    }
    for mut ship in ships.iter_mut() {
        ship.finished = true;
    }
    let mut go_to_meta = |meta: MetaState, include_unfade: bool| {
        screen_effect.queue_effect(ScreenEffect::FadeToBlack(Some(GameState {
            meta,
            pause: None,
        })));
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

use super::{menu_asset::MenuAssetComponent, title_screen::ColorMarker};
use crate::{
    cutscenes::{ChapterOneCutscenes, Cutscene, StartCutscene},
    drawing::effects::{EffectVal, Sizeable, TriggerZoomToBlack},
    environment::background::BgOffset,
    math::Spleen,
    meta::game_state::{GameState, LevelState, MenuState, MetaState, SetGameState},
    when_becomes_false, when_becomes_true,
};
use bevy::prelude::*;

const CONSTELLATION_SCREEN_RON_PATH: &'static str = "menus/constellation_screen.ron";

#[derive(Component)]
struct ConstellationScreenDeath {
    pub timer: Timer,
}

fn setup_constellation_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    MenuAssetComponent::spawn(
        &asset_server,
        &mut commands,
        "menus/constellation_screen.ron".to_string(),
    );
    commands.spawn(ConstellationScreenData { selection: -1 });
}

#[derive(Component, Debug)]
pub struct ConstellationScreenData {
    pub selection: i32,
}

fn update_constellation_screen(
    mut commands: Commands,
    mut screen_data: Query<&mut ConstellationScreenData>,
    texts: Query<(
        Entity,
        &Text,
        &Sizeable,
        Option<&EffectVal<{ Sizeable::id() }>>,
    )>,
    mut death: Query<(Entity, &mut ConstellationScreenDeath)>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut gs_writer: EventWriter<SetGameState>,
    mut ztb_writer: EventWriter<TriggerZoomToBlack>,
    mut cutscene_writer: EventWriter<StartCutscene>,
    mut bgs: Query<(&mut Sprite, &mut BgOffset, &ColorMarker)>,
) {
    let transition_time = 0.75;
    // Sparkle
    let (mut left_val, mut right_val) = (1.0, 1.0);
    for (_, text, _, val) in texts.iter() {
        if text.sections[0].value == "A".to_string() {
            left_val = match val {
                Some(f) => f.interp(),
                None => 1.0,
            };
        }
        if text.sections[0].value == "B".to_string() {
            right_val = match val {
                Some(f) => f.interp(),
                None => 1.0,
            };
        }
    }
    for (mut sprite, mut offset, og_color) in bgs.iter_mut() {
        if offset.placed.x < 0.0 {
            offset.tweak_scale = Some(left_val);
            sprite.color = Color::hsla(
                og_color.0.h(),
                og_color.0.s(),
                og_color.0.l() + left_val - 1.0,
                og_color.0.a(),
            )
        } else {
            offset.tweak_scale = Some(right_val);
            sprite.color = Color::hsla(
                og_color.0.h(),
                og_color.0.s(),
                og_color.0.l() + right_val - 1.0,
                og_color.0.a(),
            )
        }
    }
    match death.get_single_mut() {
        Err(_) => {
            // Player has not yet selected a save file
            let mut screen_data = screen_data.single_mut();
            let mut start_effect = false;
            if keys.pressed(KeyCode::ArrowLeft) {
                start_effect = screen_data.selection != 0;
                screen_data.selection = 0;
            } else if keys.pressed(KeyCode::ArrowRight) {
                start_effect = screen_data.selection != 1;
                screen_data.selection = 1;
            }
            if start_effect {
                for (id, text, sizeable, _) in texts.iter() {
                    let shared_spleen = Spleen::EaseInOutCubic;
                    let shared_duration = 0.5;
                    let shared_big = 1.3;
                    if &text.sections[0].value == "A" {
                        let goal_val = if screen_data.selection == 0 {
                            shared_big
                        } else {
                            1.0
                        };
                        sizeable.start_effect(
                            id,
                            &mut commands,
                            goal_val,
                            shared_spleen,
                            shared_duration,
                        );
                    }
                    if &text.sections[0].value == "B" {
                        let goal_val = if screen_data.selection == 1 {
                            shared_big
                        } else {
                            1.0
                        };
                        sizeable.start_effect(
                            id,
                            &mut commands,
                            goal_val,
                            shared_spleen,
                            shared_duration,
                        );
                    }
                }
            }
            if keys.pressed(KeyCode::Enter) && screen_data.selection >= 0 {
                commands.spawn(ConstellationScreenDeath {
                    timer: Timer::from_seconds(transition_time + 0.05, TimerMode::Once),
                });
                ztb_writer.send(TriggerZoomToBlack((1.0, transition_time)));
            }
        }
        Ok((id, mut death)) => {
            death.timer.tick(time.delta());
            if death.timer.finished() {
                commands.entity(id).despawn_recursive();
                // TODO: If there is >0% completion, should go to galaxy overworld
                // otherwise, should go to the first cutscene
                // gs_writer.send(SetGameState(GameState {
                //     meta: MetaState::Menu(MenuState::GalaxyOverworld),
                // }));
                gs_writer.send(SetGameState(GameState {
                    meta: MetaState::Level(LevelState::fresh_from_id("L1".to_string())),
                }));
                cutscene_writer.send(StartCutscene(Cutscene::One(ChapterOneCutscenes::Alarm)));
                ztb_writer.send(TriggerZoomToBlack((0.0, transition_time)));
            }
        }
    };
}

fn destroy_constellation_screen(
    mut commands: Commands,
    mac: Query<(Entity, &MenuAssetComponent)>,
    data: Query<Entity, With<ConstellationScreenData>>,
) {
    for (id, mac) in mac.iter() {
        if mac.path != CONSTELLATION_SCREEN_RON_PATH.to_string() {
            continue;
        }
        commands.entity(id).despawn_recursive();
    }
    for id in data.iter() {
        commands.entity(id).despawn_recursive();
    }
}

fn is_in_constellation_screen_helper(gs: &GameState) -> bool {
    match gs.meta {
        MetaState::Menu(menu_state) => match menu_state {
            MenuState::ConstellationSelect => true,
            _ => false,
        },
        _ => false,
    }
}
fn is_in_constellation_screen(gs: Res<GameState>) -> bool {
    is_in_constellation_screen_helper(&gs)
}
when_becomes_true!(
    is_in_constellation_screen_helper,
    entered_constellation_screen
);
when_becomes_false!(is_in_constellation_screen_helper, left_constellation_screen);

pub fn register_constellation_screen(app: &mut App) {
    app.add_systems(
        Update,
        setup_constellation_screen.run_if(entered_constellation_screen),
    );
    app.add_systems(
        Update,
        destroy_constellation_screen.run_if(left_constellation_screen),
    );
    app.add_systems(
        Update,
        update_constellation_screen
            .run_if(is_in_constellation_screen)
            .after(setup_constellation_screen),
    );
}

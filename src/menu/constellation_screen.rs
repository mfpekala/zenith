use crate::{
    drawing::{
        animation::{AnimationManager, SpriteInfo},
        effects::{ScreenEffect, ScreenEffectManager},
        layering::menu_layer_u8,
    },
    meta::game_state::{GameState, LevelState, MenuState, MetaState, SetGameState},
    when_becomes_false, when_becomes_true,
};
use bevy::prelude::*;

use super::placement::{GameRelativePlacement, GameRelativePlacementBundle};

/// Root of the constellation screen. Destroyed on on_destroy
#[derive(Component)]
struct ConstellationScreenRoot;

#[derive(Component)]
struct ConstellationScreenDeath {
    pub timer: Timer,
}

#[derive(Component, Debug)]
pub struct ConstellationScreenData {
    pub selection: i32,
}

#[derive(Component)]
pub struct ConstellationScreenOption(pub i32);

fn setup_constellation_screen(mut commands: Commands) {
    commands
        .spawn((
            SpatialBundle::default(),
            Name::new("constellation_menu_root"),
            ConstellationScreenRoot,
        ))
        .with_children(|parent| {
            parent.spawn(ConstellationScreenData { selection: -1 });
            // Spawn the instruction text
            let mut zenith_man = AnimationManager::single_static(SpriteInfo {
                path: "sprites/menu/constellation/use.png".to_string(),
                size: UVec2::new(357, 16),
            });
            zenith_man.set_render_layers(vec![menu_layer_u8()]);
            parent.spawn((
                GameRelativePlacementBundle::new(IVec3::new(0, -60, 0), 0.6),
                zenith_man,
            ));
            // Spawn the instruction text
            let mut left_man = AnimationManager::single_static(SpriteInfo {
                path: "sprites/menu/constellation/A.png".to_string(),
                size: UVec2::new(15, 21),
            });
            left_man.set_render_layers(vec![menu_layer_u8()]);
            parent.spawn((
                GameRelativePlacementBundle::new(IVec3::new(-80, 0, 0), 1.25),
                ConstellationScreenOption(0),
                left_man,
            ));
            let mut right_man = AnimationManager::single_static(SpriteInfo {
                path: "sprites/menu/constellation/B.png".to_string(),
                size: UVec2::new(15, 21),
            });
            right_man.set_render_layers(vec![menu_layer_u8()]);
            parent.spawn((
                GameRelativePlacementBundle::new(IVec3::new(80, 0, 0), 1.25),
                ConstellationScreenOption(1),
                right_man,
            ));
        });
}

fn update_constellation_screen(
    mut commands: Commands,
    mut screen_data: Query<&mut ConstellationScreenData>,
    mut death: Query<(Entity, &mut ConstellationScreenDeath)>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut gs_writer: EventWriter<SetGameState>,
    mut options: Query<(&ConstellationScreenOption, &mut GameRelativePlacement)>,
    mut screen_effect: ResMut<ScreenEffectManager>,
) {
    let transition_time = 1.0;
    match death.get_single_mut() {
        Err(_) => {
            // Player has not yet selected a save file
            let mut screen_data = screen_data.single_mut();
            if keys.pressed(KeyCode::ArrowLeft) {
                screen_data.selection = 0;
            } else if keys.pressed(KeyCode::ArrowRight) {
                screen_data.selection = 1;
            }
            for (option, mut placement) in options.iter_mut() {
                if screen_data.selection == option.0 {
                    placement.scale = 2.0;
                } else {
                    placement.scale = 1.2;
                }
            }
            if keys.pressed(KeyCode::Enter) && screen_data.selection >= 0 {
                screen_effect.queue_effect(ScreenEffect::FadeToBlack);
                commands.spawn(ConstellationScreenDeath {
                    timer: Timer::from_seconds(transition_time + 0.05, TimerMode::Once),
                });
            }
        }
        Ok((id, mut death)) => {
            death.timer.tick(time.delta());
            if death.timer.finished() {
                commands.entity(id).despawn_recursive();
                // TODO: If there is >0% completion, should go to galaxy overworld
                gs_writer.send(SetGameState(GameState {
                    meta: MetaState::Level(LevelState::fresh_from_id("L1".to_string())),
                }));
            }
        }
    };
}

fn destroy_constellation_screen(
    mut commands: Commands,
    root: Query<Entity, With<ConstellationScreenRoot>>,
) {
    for id in root.iter() {
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

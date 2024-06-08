use crate::{
    drawing::{
        animation::{AnimationManager, AnimationNode, SpriteInfo},
        layering::menu_layer_u8,
    },
    environment::background::{BackgroundKind, BgOffset, BgOffsetSpleen},
    math::Spleen,
    meta::game_state::{GameState, MenuState, MetaState, SetGameState},
    when_becomes_false, when_becomes_true,
};
use bevy::prelude::*;

use super::placement::GameRelativePlacementBundle;

/// Root of the title screen, will be destroyed on destroy
#[derive(Component)]
struct TitleScreenRoot;

#[derive(Component)]
struct TitleScreenDeath {
    pub timer: Timer,
}

fn setup_title_screen(mut commands: Commands, mut bg_kind: ResMut<BackgroundKind>) {
    *bg_kind = BackgroundKind::ParallaxStars(500);
    commands
        .spawn((
            SpatialBundle::default(),
            Name::new("title_screen_root"),
            TitleScreenRoot,
        ))
        .with_children(|parent| {
            // ZENITH
            let mut zenith_man = AnimationManager::single_static(SpriteInfo {
                path: "sprites/menu/title/ZENITH.png".to_string(),
                size: UVec2::new(185, 53),
            });
            zenith_man.set_render_layers(vec![menu_layer_u8()]);
            parent.spawn((
                GameRelativePlacementBundle::new(IVec3::new(0, 40, 0), 0.75),
                zenith_man,
            ));
            // Press any button to start
            let mut press_man = AnimationManager::from_nodes(vec![(
                "press",
                AnimationNode {
                    sprite: SpriteInfo {
                        path: "sprites/menu/title/press.png".to_string(),
                        size: UVec2::new(260, 14),
                    },
                    length: 2,
                    next: None,
                    pace: Some(12),
                },
            )]);
            press_man.set_render_layers(vec![menu_layer_u8()]);
            parent.spawn((
                GameRelativePlacementBundle::new(IVec3::new(0, -50, 0), 0.75),
                press_man,
            ));
        });
}

fn update_title_screen(
    mut commands: Commands,
    mut death: Query<(Entity, &mut TitleScreenDeath)>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut gs_writer: EventWriter<SetGameState>,
    bgs: Query<(Entity, &BgOffset)>,
) {
    let transition_time = 0.75;
    if keys.is_changed() && !keys.is_added() && death.iter().len() == 0 {
        for (id, offset) in bgs.iter() {
            commands.entity(id).remove::<BgOffsetSpleen>();
            commands.entity(id).insert(BgOffsetSpleen {
                vel_start: offset.vel,
                vel_goal: Vec2::new(2.0, 0.2) * 2_010.4,
                timer: Timer::from_seconds(transition_time, TimerMode::Once),
                spleen: Spleen::EaseInQuintic,
            });
        }
        commands.spawn(TitleScreenDeath {
            timer: Timer::from_seconds(transition_time + 0.25, TimerMode::Once),
        });
    }
    let Ok((id, mut death)) = death.get_single_mut() else {
        return;
    };
    death.timer.tick(time.delta());
    if death.timer.finished() {
        for (id, offset) in bgs.iter() {
            commands.entity(id).remove::<BgOffsetSpleen>();
            commands.entity(id).insert(BgOffsetSpleen {
                vel_start: offset.vel,
                vel_goal: Vec2::ZERO,
                timer: Timer::from_seconds(transition_time, TimerMode::Once),
                spleen: Spleen::EaseOutQuintic,
            });
        }
        commands.entity(id).despawn_recursive();
        gs_writer.send(SetGameState(GameState {
            meta: MetaState::Menu(MenuState::ConstellationSelect),
        }));
    }
}

fn destroy_title_screen(mut commands: Commands, markers: Query<Entity, With<TitleScreenRoot>>) {
    for eid in markers.iter() {
        commands.entity(eid).despawn_recursive();
    }
}

fn is_in_title_screen_helper(gs: &GameState) -> bool {
    match gs.meta {
        MetaState::Menu(menu_state) => match menu_state {
            MenuState::Title => true,
            _ => false,
        },
        _ => false,
    }
}
fn is_in_title_screen(gs: Res<GameState>) -> bool {
    is_in_title_screen_helper(&gs)
}
when_becomes_true!(is_in_title_screen_helper, entered_title_screen);
when_becomes_false!(is_in_title_screen_helper, left_title_screen);

pub fn register_title_screen(app: &mut App) {
    app.add_systems(Update, setup_title_screen.run_if(entered_title_screen));
    app.add_systems(Update, destroy_title_screen.run_if(left_title_screen));
    app.add_systems(Update, update_title_screen.run_if(is_in_title_screen));
}

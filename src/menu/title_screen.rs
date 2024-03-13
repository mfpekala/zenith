use super::menu_asset::MenuAssetComponent;
use crate::{
    environment::background::{HyperSpace, BASE_TITLE_HYPERSPACE_SPEED, MAX_HYPERSPACE_SPEED},
    math::Spleen,
    meta::game_state::{GameState, MenuState, MetaState, SetGameState},
    when_becomes_false, when_becomes_true,
};
use bevy::prelude::*;

const TITLE_SCREEN_RON_PATH: &'static str = "menus/title_screen.ron";

#[derive(Component)]
struct TitleScreenDeath {
    pub timer: Timer,
}

fn setup_title_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    MenuAssetComponent::spawn(
        asset_server,
        &mut commands,
        TITLE_SCREEN_RON_PATH.to_string(),
    );
}

fn update_title_screen(
    mut commands: Commands,
    mut death: Query<(Entity, &mut TitleScreenDeath)>,
    time: Res<Time>,
    mut hyperspace: ResMut<HyperSpace>,
    keys: Res<ButtonInput<KeyCode>>,
    mut gs_writer: EventWriter<SetGameState>,
) {
    let transition_time = 0.75;
    if keys.is_changed() && !keys.is_added() && death.iter().len() == 0 {
        hyperspace.approach_speed(
            BASE_TITLE_HYPERSPACE_SPEED * MAX_HYPERSPACE_SPEED,
            transition_time,
            Spleen::EaseInQuintic,
        );
        commands.spawn(TitleScreenDeath {
            timer: Timer::from_seconds(transition_time + 0.05, TimerMode::Once),
        });
    }
    let Ok((id, mut death)) = death.get_single_mut() else {
        return;
    };
    death.timer.tick(time.delta());
    if death.timer.finished() {
        hyperspace.approach_speed(IVec2::ZERO, transition_time * 1.5, Spleen::EaseOutQuintic);
        commands.entity(id).despawn_recursive();
        gs_writer.send(SetGameState(GameState {
            meta: MetaState::Menu(MenuState::ConstellationSelect),
        }));
    }
}

fn destroy_title_screen(mut commands: Commands, mac: Query<(Entity, &MenuAssetComponent)>) {
    for (id, mac) in mac.iter() {
        if mac.path != TITLE_SCREEN_RON_PATH.to_string() {
            continue;
        }
        commands.entity(id).despawn_recursive();
        for curse in mac.cursed_children.iter() {
            commands.entity(*curse).despawn_recursive();
        }
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

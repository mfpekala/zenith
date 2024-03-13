use bevy::prelude::*;

use crate::{
    environment::background::{HyperSpace, BASE_TITLE_HYPERSPACE_SPEED},
    math::Spleen,
    meta::game_state::{pretranslate_events, GameState, MenuState, MetaState},
    when_becomes_false, when_becomes_true,
};

use super::menu_asset::MenuAssetComponent;

#[derive(Component)]
struct ConstellationScreenDeath {
    pub timer: Timer,
}

fn setup_constellation_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("setup constellation");
    MenuAssetComponent::spawn(
        asset_server,
        &mut commands,
        "menus/constellation_screen.ron".to_string(),
    );
}

#[derive(Component)]
struct ConstellationScreenData {
    pub selection: i32,
}

fn update_constellation_screen(
    mut commands: Commands,
    mut death: Query<(Entity, &mut ConstellationScreenDeath)>,
    time: Res<Time>,
    mut hyperspace: ResMut<HyperSpace>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let transition_time = 0.75;
    if keys.is_changed() && !keys.is_added() && death.iter().len() == 0 {
        hyperspace.approach_speed(
            BASE_TITLE_HYPERSPACE_SPEED * 20_000,
            transition_time,
            Spleen::EaseInQuintic,
        );
        commands.spawn(ConstellationScreenDeath {
            timer: Timer::from_seconds(transition_time + 0.05, TimerMode::Once),
        });
    }
    let Ok((id, mut death)) = death.get_single_mut() else {
        return;
    };
    death.timer.tick(time.delta());
    if death.timer.finished() {
        hyperspace.approach_speed(
            BASE_TITLE_HYPERSPACE_SPEED,
            transition_time,
            Spleen::EaseOutQuintic,
        );
        commands.entity(id).despawn_recursive();
    }
}

fn destroy_constellation_screen(
    mut commands: Commands,
    mac: Query<Entity, With<MenuAssetComponent>>,
) {
    for id in mac.iter() {
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
        setup_constellation_screen
            .run_if(entered_constellation_screen)
            .after(pretranslate_events),
    );
    app.add_systems(
        Update,
        destroy_constellation_screen
            .run_if(left_constellation_screen)
            .after(pretranslate_events),
    );
    app.add_systems(
        Update,
        update_constellation_screen.run_if(is_in_constellation_screen),
    );
}

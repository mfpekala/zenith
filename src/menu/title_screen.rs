use bevy::prelude::*;

use crate::{
    meta::game_state::{GameState, MenuState, MetaState},
    when_becomes_false, when_becomes_true,
};

use super::menu_asset::{MenuAsset, MenuAssetComponent};

#[derive(Resource, Debug)]
pub struct TestMarker(pub Handle<MenuAsset>);

fn setup_title_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    MenuAssetComponent::spawn(
        asset_server,
        &mut commands,
        "menus/title_screen.ron".to_string(),
    );
}

fn destroy_title_screen(mut commands: Commands, mac: Query<Entity, With<MenuAssetComponent>>) {
    for id in mac.iter() {
        commands.entity(id).despawn_recursive();
    }
}

fn is_in_title_screen(gs: &GameState) -> bool {
    match gs.meta {
        MetaState::Menu(menu_state) => match menu_state {
            MenuState::Title => true,
            _ => false,
        },
        _ => false,
    }
}

when_becomes_true!(is_in_title_screen, entered_title_screen);
when_becomes_false!(is_in_title_screen, left_title_screen);

pub fn register_title_screen(app: &mut App) {
    app.add_systems(Update, setup_title_screen.run_if(entered_title_screen));
    app.add_systems(Update, destroy_title_screen.run_if(left_title_screen));
}

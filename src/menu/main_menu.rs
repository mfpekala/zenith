use bevy::prelude::*;

use super::menu_asset::{MenuAsset, MenuAssetComponent};

#[derive(Resource, Debug)]
pub struct TestMarker(pub Handle<MenuAsset>);

fn test_ron_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    // let test = asset_server.load::<MenuAsset>("menus/main_menu.ron");
    // commands.insert_resource(TestMarker(test));
    MenuAssetComponent::spawn(
        asset_server,
        &mut commands,
        "menus/main_menu.ron".to_string(),
    );
}

// fn test_ron_update(menu_assets: Res<Assets<MenuAsset>>, handles: Res<TestMarker>) {
//     // println!("handle: {:?}", handles);
//     // let got = menu_assets.get(handles.0.id());
//     // println!("got: {:?}", got);
// }

pub fn register_main_menu(app: &mut App) {
    app.add_systems(Startup, test_ron_loading);
    // app.add_systems(Update, test_ron_update);
}

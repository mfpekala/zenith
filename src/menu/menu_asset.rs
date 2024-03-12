use bevy::prelude::*;

use crate::drawing::{lightmap::sprite_layer, text::TextBox};

#[derive(
    serde::Deserialize,
    bevy::asset::Asset,
    bevy::reflect::TypePath,
    Debug,
    PartialEq,
    PartialOrd,
    Clone,
)]
pub struct MenuAsset {
    texts: Vec<TextBox>,
}
impl MenuAsset {
    pub fn spawn(&self, commands: &mut Commands, asset_server: &Res<AssetServer>) -> Vec<Entity> {
        let mut cursed = vec![];
        for tb in self.texts.iter() {
            let id = tb.spawn(commands, asset_server);
            cursed.push(id);
        }
        cursed
    }
}

#[derive(Component)]
pub struct MenuAssetComponent {
    pub handle: Handle<MenuAsset>,
    pub last_version: Option<MenuAsset>,
    pub cursed_children: Vec<Entity>,
}
impl MenuAssetComponent {
    pub fn spawn(asset_server: Res<AssetServer>, commands: &mut Commands, path: String) {
        let handle = asset_server.load::<MenuAsset>(path);
        commands.spawn(Self {
            handle,
            last_version: None,
            cursed_children: vec![],
        });
    }
}

fn render_menu_asset(
    mut commands: Commands,
    mut comp_q: Query<(Entity, &mut MenuAssetComponent)>,
    menu_assets: Res<Assets<MenuAsset>>,
    asset_server: Res<AssetServer>,
) {
    let Ok((id, mut comp)) = comp_q.get_single_mut() else {
        return;
    };
    let Some(res) = menu_assets.get(comp.handle.id()) else {
        commands.entity(id).despawn_recursive();
        return;
    };
    if comp.last_version == Some(res.clone()) {
        // Nothing's changed
        return;
    }
    // We're clearing and remaking all the children
    for child in comp.cursed_children.iter() {
        commands.entity(*child).despawn_recursive();
    }
    comp.cursed_children = res.spawn(&mut commands, &asset_server);
    comp.last_version = Some(res.clone());
}

pub struct MenuAssetPlugin;

impl Plugin for MenuAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, render_menu_asset);
    }
}

use bevy::prelude::*;

use crate::drawing::text::TextBox;

#[derive(
    serde::Deserialize,
    bevy::asset::Asset,
    bevy::reflect::TypePath,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
)]
pub struct MenuAsset {
    texts: Vec<TextBox>,
}
impl MenuAsset {
    pub fn spawn(&self, commands: &mut Commands) {
        println!("WOULD HAVE SPAWNED: {:?}", self);
    }
}

#[derive(Component)]
pub struct MenuAssetComponent {
    pub handle: Handle<MenuAsset>,
    pub last_version: Option<MenuAsset>,
}
impl MenuAssetComponent {
    pub fn spawn(asset_server: Res<AssetServer>, commands: &mut Commands, path: String) {
        let handle = asset_server.load::<MenuAsset>(path);
        commands.spawn(Self {
            handle,
            last_version: None,
        });
    }
}

fn render_menu_asset(
    mut commands: Commands,
    mut comp_q: Query<(Entity, &mut MenuAssetComponent, Option<&Children>)>,
    menu_assets: Res<Assets<MenuAsset>>,
) {
    let Ok((id, mut comp, children)) = comp_q.get_single_mut() else {
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
    if let Some(children) = children {
        for child in children {
            commands.entity(id).remove_children(&[*child]);
            commands.entity(*child).despawn_recursive();
        }
    }
    res.spawn(&mut commands);
    comp.last_version = Some(res.clone());
}

pub struct MenuAssetPlugin;

impl Plugin for MenuAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, render_menu_asset);
    }
}

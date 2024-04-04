use crate::environment::background::BackgroundPlugin;

use self::{
    animated::MyAnimationPlugin,
    bordered_mesh::bordered_mesh_trickle_down,
    effects::EffectsPlugin,
    layering::{bg_light_layer, bg_sprite_layer, light_layer, sprite_layer},
    light::register_light,
    mesh::scroll_sprite_materials,
    mesh_head::{resolve_mesh_head_stubs, update_mesh_heads},
    sprite_head::{resolve_sprite_head_stubs, update_sprite_heads},
    sprite_mat::SpriteMaterialPlugin,
    text::ZenithTextPlugin,
};
use bevy::prelude::*;

pub mod animated;
pub mod bordered_mesh;
pub mod effects;
pub mod layering;
pub mod light;
pub mod mesh;
pub mod mesh_head;
pub mod post_pixel;
pub mod sprite_head;
pub mod sprite_mat;
pub mod sunrise_mat;
pub mod text;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BgSpriteGizmoGroup;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BgLightGizmoGroup;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct LightGizmoGroup;

pub fn setup_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let config = config_store.config_mut::<DefaultGizmoConfigGroup>().0;
    config.line_width = 2.0;
    config.render_layers = sprite_layer();

    let config = config_store.config_mut::<LightGizmoGroup>().0;
    config.line_width = 2.0;
    config.render_layers = light_layer();

    let config = config_store.config_mut::<BgSpriteGizmoGroup>().0;
    config.line_width = 2.0;
    config.render_layers = bg_sprite_layer();

    let config = config_store.config_mut::<BgLightGizmoGroup>().0;
    config.line_width = 2.0;
    config.render_layers = bg_light_layer();
}

pub fn register_drawing(app: &mut App) {
    app.add_plugins(BackgroundPlugin);
    app.add_plugins(EffectsPlugin);
    app.add_plugins(MyAnimationPlugin);
    app.add_plugins(SpriteMaterialPlugin);
    app.add_plugins(ZenithTextPlugin);

    app.add_systems(Startup, setup_gizmos);

    app.init_gizmo_group::<LightGizmoGroup>();
    app.init_gizmo_group::<BgSpriteGizmoGroup>();
    app.init_gizmo_group::<BgLightGizmoGroup>();

    app.add_systems(FixedUpdate, scroll_sprite_materials);
    app.add_systems(Update, bordered_mesh_trickle_down);

    app.add_systems(Update, (resolve_sprite_head_stubs, update_sprite_heads));
    app.add_systems(Update, (resolve_mesh_head_stubs, update_mesh_heads));

    register_light(app);
}

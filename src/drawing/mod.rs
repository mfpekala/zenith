use crate::environment::background::BackgroundPlugin;

use self::{
    animation::GoatedAnimationPlugin,
    effects::EffectsPlugin,
    layering::{bg_light_layer, bg_sprite_layer, light_layer, sprite_layer},
    text::ZenithTextPlugin,
};
use bevy::prelude::*;

pub mod animation;
pub mod animation_mat;
pub mod bordered_mesh;
pub mod effects;
pub mod layering;
pub mod light;
pub mod mesh;
pub mod post_pixel;
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
    app.add_plugins(GoatedAnimationPlugin);
    app.add_plugins(ZenithTextPlugin);

    app.add_systems(Startup, setup_gizmos);

    app.init_gizmo_group::<LightGizmoGroup>();
    app.init_gizmo_group::<BgSpriteGizmoGroup>();
    app.init_gizmo_group::<BgLightGizmoGroup>();
}

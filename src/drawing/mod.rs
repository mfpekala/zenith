use self::{
    hollow::register_hollow_drawing,
    light::register_light,
    lightmap::{light_layer, sprite_layer},
    pixel_mesh::register_pixel_meshes,
};
use bevy::prelude::*;

pub mod hollow;
pub mod light;
pub mod lightmap;
pub mod mesh;
pub mod pixel_mesh;
pub mod post_pixel;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct LightGizmoGroup;

pub fn setup_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let config = config_store.config_mut::<DefaultGizmoConfigGroup>().0;
    config.line_width = 6.0;
    config.render_layers = sprite_layer();

    let config = config_store.config_mut::<LightGizmoGroup>().0;
    config.line_width = 6.0;
    config.render_layers = light_layer();
}

pub fn register_drawing(app: &mut App) {
    app.add_systems(Startup, setup_gizmos);
    app.init_gizmo_group::<LightGizmoGroup>();
    register_hollow_drawing(app);
    register_light(app);
    register_pixel_meshes(app);
}

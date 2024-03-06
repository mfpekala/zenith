use self::{hollow::register_hollow_drawing, lightmap::sprite_layer};
use bevy::prelude::*;

pub mod hollow;
pub mod light;
pub mod lightmap;
pub mod mesh;
pub mod post_pixel;

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct MyGizmoGroup;

pub fn setup_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let config = config_store.config_mut::<DefaultGizmoConfigGroup>().0;
    config.line_width = 6.0;
    config.render_layers = sprite_layer();
}

pub fn register_drawing(app: &mut App) {
    app.add_systems(Startup, setup_gizmos);
    register_hollow_drawing(app);
}

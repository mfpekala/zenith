use bevy::{prelude::*, text::TextLayoutInfo};

use crate::{
    drawing::{
        layering::menu_layer,
        text::{TextAlign, TextBoxBundle, TextWeight},
    },
    meta::consts::MENU_GROWTH,
};

use super::placement::{self, GameRelativePlacement, GameRelativePlacementBundle};

/// Spawn invisible
/// Get the dim of the text
/// Update the size of the button background
/// Show
struct RustShutup;

/// Sends the id
#[derive(Event)]
pub(super) struct MenuButtonPressedEvent(pub String);

#[derive(Component)]
pub(super) struct MenuButton {
    id: String,
    text: String,
    enabled: bool,
    size: f32,
    scale: f32,
    idle_color: Color,
    hover_color: Color,
    pressed_color: Color,
}
impl MenuButton {
    pub(super) fn basic(id: &str, text: &str) -> Self {
        Self {
            id: id.to_string(),
            text: text.to_string(),
            enabled: true,
            size: 36.0,
            scale: 0.25,
            idle_color: Color::WHITE,
            hover_color: Color::YELLOW,
            pressed_color: Color::RED,
        }
    }
}

#[derive(Component)]
pub(super) struct MenuButtonInnerSpawnMarker;

#[derive(Bundle)]
pub(super) struct MenuButtonBundle {
    button: MenuButton,
    placement: GameRelativePlacementBundle,
    name: Name,
}
impl MenuButtonBundle {
    pub(super) fn new(button: MenuButton, placement: GameRelativePlacement) -> Self {
        Self {
            name: Name::new(format!("menu_button_parent_{}", button.id.clone())),
            button,
            placement: GameRelativePlacementBundle::from_inner(placement),
        }
    }
}

pub(super) fn materialize_buttons(
    mut commands: Commands,
    buttons: Query<(Entity, &MenuButton), Without<Children>>,
    asset_server: Res<AssetServer>,
) {
    for (eid, button_info) in buttons.iter() {
        let mut text_bundle = (
            TextBoxBundle::new_menu_text(
                &button_info.text,
                button_info.size,
                GameRelativePlacement::new(IVec3::new(0, 0, 1), button_info.scale),
                Color::BLACK,
                TextWeight::Regular,
                TextAlign::Center,
                &asset_server,
            ),
            Name::new(button_info.id.clone()),
            MenuButtonInnerSpawnMarker,
        );
        text_bundle.0 .0.inner.visibility = Visibility::Hidden;
        commands.entity(eid).with_children(|parent| {
            parent.spawn(text_bundle);
        });
    }
}

pub(super) fn materialize_button_backgrounds(
    mut commands: Commands,
    button_parents: Query<&MenuButton>,
    mut button_inner: Query<
        (Entity, &Parent, &TextLayoutInfo, &mut Visibility),
        With<MenuButtonInnerSpawnMarker>,
    >,
) {
    for (child_eid, parent, layout_info, mut viz) in button_inner.iter_mut() {
        if layout_info.logical_size.x == 0.0 {
            continue;
        };
        commands
            .entity(child_eid)
            .remove::<MenuButtonInnerSpawnMarker>();
        *viz = Visibility::Inherited;
        let Ok(button_info) = button_parents.get(parent.get()) else {
            continue;
        };
        commands.entity(parent.get()).with_children(|parent| {
            let bund = (
                SpriteBundle {
                    sprite: Sprite {
                        color: button_info.idle_color,
                        ..default()
                    },
                    transform: Transform {
                        scale: Vec3::new(
                            layout_info.logical_size.x,
                            layout_info.logical_size.y,
                            1.0,
                        ) * MENU_GROWTH as f32
                            * button_info.scale,
                        ..default()
                    },
                    ..default()
                },
                menu_layer(),
            );
            parent.spawn(bund);
        });
    }
}

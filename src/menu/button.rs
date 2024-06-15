use bevy::{prelude::*, text::TextLayoutInfo};

use crate::{
    drawing::{
        layering::menu_layer,
        text::{TextAlign, TextBoxBundle, TextWeight},
    },
    input::MouseState,
    meta::consts::MENU_GROWTH,
};

use super::placement::{GameRelativePlacement, GameRelativePlacementBundle};

/// Sends the id
#[derive(Event)]
pub(super) struct MenuButtonPressed(pub String);

#[derive(Component)]
pub(super) struct MenuButton {
    id: String,
    text: String,
    _enabled: bool,
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
            _enabled: true,
            size: 24.0,
            scale: 0.15,
            idle_color: Color::WHITE,
            hover_color: Color::YELLOW,
            pressed_color: Color::RED,
        }
    }
}

#[derive(Component)]
pub(super) struct MenuButtonInnerSpawnMarker;

#[derive(Component)]
pub(super) struct MenuButtonBorder;

/// TODO: This is a log of data dup with the MenuButton, because querying the hierarchy is hard
/// For instance, the data we need is in the grandparent of the fill, so updating state and color is hard
/// It feels like there just isn't a good way to do this in bevy
/// Maybe a PR?
#[derive(Component)]
pub(super) struct MenuButtonFill {
    is_hovered: bool,
    is_pressed: bool,
    idle_color: Color,
    hover_color: Color,
    pressed_color: Color,
    id: String,
}

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
        let adjusted_size = Vec3::new(
            layout_info.logical_size.x + 2.0,
            layout_info.logical_size.y * 0.75,
            1.0,
        );
        let correct_tran = Vec2::new(-1.0, -layout_info.logical_size.y / 8.0);
        let rect_bund = |color: Color, scale: Vec3, z: i32| {
            (
                SpriteBundle {
                    sprite: Sprite { color, ..default() },
                    transform: Transform {
                        scale: scale * MENU_GROWTH as f32 * button_info.scale,
                        translation: correct_tran.extend(z as f32),
                        ..default()
                    },
                    ..default()
                },
                menu_layer(),
            )
        };
        commands.entity(parent.get()).with_children(|parent| {
            let border = (
                rect_bund(Color::BLACK, adjusted_size, 0),
                MenuButtonBorder,
                Name::new("border"),
            );
            let fill_size = adjusted_size - Vec3::new(2.0, 2.0, 0.0);
            let fill = (
                rect_bund(button_info.idle_color, fill_size, 1),
                MenuButtonFill {
                    is_hovered: false,
                    is_pressed: false,
                    idle_color: button_info.idle_color,
                    hover_color: button_info.hover_color,
                    pressed_color: button_info.pressed_color,
                    id: button_info.id.clone(),
                },
                Name::new("fill"),
            );
            parent.spawn(border);
            parent.spawn(fill);
        });
    }
}

pub(super) fn update_button_state(
    mut fills: Query<(&mut MenuButtonFill, &Transform, &GlobalTransform)>,
    mouse_state: Res<MouseState>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut pressed_writer: EventWriter<MenuButtonPressed>,
) {
    for (mut fill, tran, gtran) in fills.iter_mut() {
        let mut clean_pos = mouse_state.pos.as_vec2();
        clean_pos.y *= -1.0;
        let diff = clean_pos - gtran.translation().truncate();
        let x_in = diff.x.abs() * 2.0 < tran.scale.x * MENU_GROWTH as f32;
        let y_in = diff.y.abs() * 2.0 < tran.scale.y * MENU_GROWTH as f32;
        let is_over = x_in && y_in;
        if fill.is_pressed {
            if !is_over {
                fill.is_pressed = false;
                fill.is_hovered = false;
            } else if mouse_buttons.just_released(MouseButton::Left) {
                fill.is_pressed = false;
                fill.is_hovered = false;
                pressed_writer.send(MenuButtonPressed(fill.id.clone()));
            }
        } else {
            if is_over && mouse_buttons.just_pressed(MouseButton::Left) {
                fill.is_pressed = true;
                fill.is_hovered = false;
            } else if is_over {
                fill.is_hovered = true;
            } else {
                fill.is_hovered = false;
            }
        }
    }
}

pub(super) fn update_button_fill_colors(mut fills: Query<(&mut Sprite, &MenuButtonFill)>) {
    for (mut sprite, fill) in fills.iter_mut() {
        if fill.is_pressed {
            sprite.color = fill.pressed_color;
        } else if fill.is_hovered {
            sprite.color = fill.hover_color;
        } else {
            sprite.color = fill.idle_color;
        }
    }
}

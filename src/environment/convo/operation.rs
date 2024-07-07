use std::time::Duration;

use bevy::{prelude::*, render::view::RenderLayers, text::Text2dBounds};

use crate::{
    camera::CameraMarker,
    drawing::{animation::AnimationManager, layering::menu_layer, text::TextWeight},
    math::Spleen,
    meta::consts::MENU_GROWTH,
    physics::dyno::IntMoveable,
    sound::effect::SoundEffect,
};

use super::{
    data::{in_convo, ConvoKind},
    CameraBeforeConvo, Convo, ConvoBoxBundle, ConvoBoxContent, ConvoBoxPos, ConvoBoxProgress,
    ConvoEnded, StartConvo,
};

#[derive(Component)]
struct MaterializedMain;

#[derive(Component)]
struct MaterializedBackground;
#[derive(Bundle)]
struct MaterializedBackgroundBundle {
    name: Name,
    marker: MaterializedBackground,
    sprite: SpriteBundle,
    render_layers: RenderLayers,
}

struct BackgroundHelperInfo {
    text_offset: Vec2,
    text_bounds: Vec2,
    portrait_offset: Option<Vec2>,
}

impl MaterializedBackgroundBundle {
    /// NOTE: Size is ASSUMED to already have been scaled by MENU_GROWTH as needed
    /// NOTE: Returns the bundle AND the text offset/bounds which should be given to the text
    fn from_pos(
        pos: &ConvoBoxPos,
        asset_server: &Res<AssetServer>,
    ) -> (Self, BackgroundHelperInfo) {
        match pos {
            ConvoBoxPos::Default => {
                let mgf = MENU_GROWTH as f32;
                let text_offset = Vec2::new(-160.0 / 2.0 + 34.0, 36.0 / 2.0 - 6.0) * mgf;
                let text_bounds = Vec2::new(120.0, 24.0) * mgf;
                let portrait_offset = Vec2::new(-80.0 + 18.0, 0.0) * mgf;
                // Why this? We're using the built-in (ass) Sprite, so it'll be outside it's tran
                let bg_size_tran = Vec2::ONE * mgf;
                (
                    Self {
                        name: Name::new("background"),
                        marker: MaterializedBackground,
                        sprite: SpriteBundle {
                            texture: asset_server.load("sprites/convo/background.png"),
                            transform: Transform::from_scale(bg_size_tran.extend(1.0))
                                .with_translation(Vec2::ZERO.extend(-1.0)),
                            ..default()
                        },
                        render_layers: menu_layer(),
                    },
                    BackgroundHelperInfo {
                        text_offset,
                        text_bounds,
                        portrait_offset: Some(portrait_offset),
                    },
                )
            }
        }
    }
}

#[derive(Component)]
struct MaterializedText;
#[derive(Bundle)]
struct MaterializedTextBundle {
    name: Name,
    marker: MaterializedText,
    text: Text2dBundle,
    render_layers: RenderLayers,
}
impl MaterializedTextBundle {
    /// NOTE: Bounds is ASSUMED to already have been scaled by MENU_GROWTH as needed
    fn from_offset_n_bounds(
        offset: Vec2,
        bounds: Vec2,
        content: String,
        asset_server: &Res<AssetServer>,
    ) -> Self {
        Self {
            name: Name::new("text"),
            marker: MaterializedText,
            text: Text2dBundle {
                text: Text::from_section(
                    content,
                    TextStyle {
                        font: TextWeight::Regular.to_handle_ass(asset_server),
                        font_size: 60.0,
                        color: Color::ANTIQUE_WHITE,
                        ..default()
                    },
                )
                .with_justify(JustifyText::Left),
                text_2d_bounds: Text2dBounds { size: bounds },
                text_anchor: bevy::sprite::Anchor::TopLeft,
                transform: Transform::from_translation(offset.extend(0.0)),
                ..default()
            },
            render_layers: menu_layer(),
        }
    }
}

/// See ./data/speaker.rs for spawning info
#[derive(Component)]
pub struct MaterializedPortrait;
#[derive(Bundle)]
pub struct MaterializedPortraitBundle {
    pub name: Name,
    pub marker: MaterializedPortrait,
    pub anim: AnimationManager,
    pub spatial: SpatialBundle,
}

/// See ./data/speaker.rs for spawning info
#[derive(Component)]
pub struct MaterializedSound;
#[derive(Bundle)]
pub struct MaterializedSoundBundle {
    pub name: Name,
    pub marker: MaterializedSound,
    pub sound: SoundEffect,
}

#[derive(Bundle)]
pub(super) struct MaterializedBundle {
    name: Name,
    main: MaterializedMain,
    partial: ConvoBoxBundle,
    spatial: SpatialBundle,
}
impl MaterializedBundle {
    pub(super) fn spawn(
        parent_eid: Entity,
        commands: &mut Commands,
        partial: ConvoBoxBundle,
        asset_server: &Res<AssetServer>,
    ) -> Entity {
        let mut id = Entity::PLACEHOLDER;
        commands.entity(parent_eid).with_children(|meta_parent| {
            // Extract the info about what we'll be spawning
            let pos = match partial.pos {
                ConvoBoxPos::Default => Vec2::new(0.0, -60.0),
            } * MENU_GROWTH as f32;

            id = meta_parent
                .spawn(MaterializedBundle {
                    name: Name::new("materialized_convo_box_bundle"),
                    main: MaterializedMain,
                    partial: partial.clone(),
                    spatial: SpatialBundle::from_transform(Transform::from_translation(
                        pos.extend(-5.0),
                    )),
                })
                .with_children(|main_parent| {
                    let (bg, bg_helper) =
                        MaterializedBackgroundBundle::from_pos(&partial.pos, &asset_server);
                    main_parent.spawn(bg);
                    main_parent.spawn(MaterializedTextBundle::from_offset_n_bounds(
                        bg_helper.text_offset,
                        bg_helper.text_bounds,
                        partial.content.content,
                        &asset_server,
                    ));
                    if let Some((portrait, sound)) =
                        partial.speaker.materialize(bg_helper.portrait_offset)
                    {
                        main_parent.spawn(portrait);
                        main_parent.spawn(sound);
                    }
                })
                .id();
        });
        id
    }
}

/// Updates the convo boxes.
fn update_box(
    mut bx: Query<(Entity, &ConvoBoxContent, &mut ConvoBoxProgress)>,
    mut text_q: Query<(&Parent, &mut Text), With<MaterializedText>>,
    time: Res<Time>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut camera_q: Query<(&mut CameraMarker, &mut IntMoveable)>,
) {
    let Ok((mid, bx_content, mut bx_progress)) = bx.get_single_mut() else {
        return;
    };
    let Ok((text_parent, mut text)) = text_q.get_single_mut() else {
        warn!("Weird stuff happpening in update_box");
        return;
    };
    if text_parent.get() != mid {
        warn!("Weird non-familial text relations in update_box");
        return;
    }

    // Handle the timer and absolute
    bx_progress.timer.tick(time.delta());
    if mouse_input.just_pressed(MouseButton::Right) {
        if bx_progress.timer.finished() {
            bx_progress.absolutely_finished = true;
        } else {
            let amount_left = bx_progress.timer.remaining_secs();
            bx_progress.timer.tick(Duration::from_secs_f32(amount_left));
        }
    }
    let frac_complete = bx_progress.timer.fraction();

    // Update text
    let num_chars_showing = (bx_content.content.len() as f32 * frac_complete).ceil() as usize;
    let substring_showing = bx_content.content[0..num_chars_showing].to_string();
    text.sections[0].value = substring_showing;

    // Move camera
    if let Some((start_pos, end_pos)) = bx_content.camera_mvmt {
        let (_, mut camera_mv) = camera_q.single_mut();
        let interped_frac = Spleen::EaseInOutCubic.interp(frac_complete);
        let interped_pos =
            start_pos.as_vec2() + interped_frac * (end_pos.as_vec2() - start_pos.as_vec2());
        camera_mv.pos.x = interped_pos.x.round() as i32;
        camera_mv.pos.y = interped_pos.y.round() as i32;
    }
    // Scale camera
    if let Some(scale) = bx_content.camera_scale {
        let (mut camera_marker, _) = camera_q.single_mut();
        camera_marker.scale = scale;
    }
}

/// Basically just checks if the current box is done. If so, spawns the next one and despawns the current one.
/// If there are no more boxes to spawn, despawns the convo and sends the finished event.
fn update_convo(
    mut commands: Commands,
    convo_root: Query<Entity, With<ConvoRoot>>,
    mut convo: Query<(Entity, &mut Convo)>,
    box_q: Query<(Entity, &ConvoBoxProgress)>,
    mut convo_ended: EventWriter<ConvoEnded>,
    asset_server: Res<AssetServer>,
    mut camera: Query<(&mut CameraMarker, &mut IntMoveable)>,
    camera_before_convo: Query<(Entity, &CameraBeforeConvo)>,
) {
    let convo_root = convo_root.single();
    let (cid, mut convo) = convo.single_mut();

    let bx = convo.active_eid.map(|eid| box_q.get(eid).unwrap());

    // Figure out if we can spawn. Also has side effect of despawining the current
    // one if it's absolutely finished.
    let can_spawn = match bx {
        Some((bid, bx)) => {
            if bx.absolutely_finished {
                commands.entity(bid).despawn_recursive();
            }
            bx.absolutely_finished
        }
        None => true,
    };

    if can_spawn {
        match convo.bundles.pop_front() {
            Some(partial) => {
                // Time to have babies
                let eid =
                    MaterializedBundle::spawn(convo_root, &mut commands, partial, &asset_server);
                convo.active_eid = Some(eid);
            }
            None => {
                // Time to die
                commands.entity(cid).despawn_recursive();
                convo_ended.send(ConvoEnded(convo.kind));
                let (mut camera_marker, mut camera_mv) = camera.single_mut();
                let (bid, CameraBeforeConvo(saved_camera_marker, saved_camera_mv)) =
                    camera_before_convo.single();
                *camera_marker = saved_camera_marker.clone();
                *camera_mv = saved_camera_mv.clone();
                commands.entity(bid).despawn();
            }
        }
    }
}

#[derive(Component)]
pub(super) struct ConvoRoot;
fn setup_convo_ops(mut commands: Commands) {
    commands.spawn((Name::new("convo_root"), ConvoRoot, SpatialBundle::default()));
}

fn test_convos(mut writer: EventWriter<StartConvo>, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        writer.send(StartConvo(ConvoKind::Test));
    }
}

pub(super) fn register_convo_ops(app: &mut App) {
    app.add_systems(Startup, setup_convo_ops);
    app.add_systems(Update, (update_box, update_convo).run_if(in_convo));

    // TESTING
    app.add_systems(Update, test_convos);
}

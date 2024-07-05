use crate::{
    drawing::{
        animation::{AnimationManager, AnimationNode, SpriteInfo},
        effects::ScreenEffectManager,
        layering::{menu_layer, menu_layer_u8},
    },
    meta::{
        consts::{MENU_GROWTH, MENU_HEIGHT, MENU_WIDTH},
        game_state::{EditingState, EditorState, GameState, MenuState, MetaState, SetMetaState},
    },
    sound::{
        effect::SoundEffect,
        music::{MusicKind, MusicManager},
    },
    when_becomes_false, when_becomes_true,
};
use bevy::prelude::*;

use super::{
    button::{MenuButton, MenuButtonBundle},
    placement::GameRelativePlacement,
    update_any_menu, MenuSettingsButton,
};

/// Root of the studio screen, will be destroyed on destroy
#[derive(Component)]
struct StudioScreenRoot {
    transition_started: bool,
}

#[derive(Component)]
struct StudioScreenAnimation;

fn setup_studio_screen(mut commands: Commands) {
    commands
        .spawn((
            SpatialBundle::default(),
            Name::new("studio_screen_root"),
            StudioScreenRoot {
                transition_started: false,
            },
        ))
        .with_children(|parent| {
            let dillo_man = AnimationManager::from_nodes(vec![
                (
                    "dyno",
                    AnimationNode {
                        sprite: SpriteInfo {
                            path: "sprites/studio/armadillo_games_dyno.png".into(),
                            size: UVec2::new(134, 48),
                            ..default()
                        },
                        length: 72,
                        next: Some("static".into()),
                        pace: Some(1),
                    },
                ),
                (
                    "static",
                    AnimationNode {
                        sprite: SpriteInfo {
                            path: "sprites/studio/armadillo_games_static.png".into(),
                            size: UVec2::new(134, 48),
                            ..default()
                        },
                        length: 1,
                        next: Some("static".into()),
                        pace: None,
                    },
                ),
            ])
            .force_render_layer(menu_layer_u8());
            parent.spawn((
                dillo_man,
                SpatialBundle::from_transform(
                    Transform::from_scale((Vec2::ONE * 2.0 * MENU_GROWTH as f32).extend(1.0))
                        .with_translation(Vec3::new(0.0, 0.0, 1.0)),
                ),
                StudioScreenAnimation,
            ));
            parent.spawn((
                Name::new("studio_white_background"),
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::WHITE,
                        ..default()
                    },
                    transform: Transform {
                        scale: Vec3::new(MENU_WIDTH as f32, MENU_HEIGHT as f32, 1.0),
                        translation: Vec3::new(0.0, 0.0, 0.0),
                        ..default()
                    },
                    ..default()
                },
                menu_layer(),
            ));
        });
    commands.spawn(SoundEffect::universal(
        "sound_effects/temp_studio.ogg",
        1.0,
        false,
    ));
}

fn update_studio_screen(
    keys: Res<ButtonInput<KeyCode>>,
    mut gs_writer: EventWriter<SetMetaState>,
    mut root: Query<&mut StudioScreenRoot>,
    anim: Query<&AnimationManager, With<StudioScreenAnimation>>,
    mut screen_effects: ResMut<ScreenEffectManager>,
) {
    let Ok(mut root) = root.get_single_mut() else {
        return;
    };
    let Ok(anim) = anim.get_single() else {
        return;
    };
    if keys.just_pressed(KeyCode::Enter) {
        if keys.pressed(KeyCode::KeyE) {
            // Activate the editor by pressing E
            gs_writer.send(SetMetaState(MetaState::Editor(EditorState::Editing(
                EditingState::blank(),
            ))));
            return;
        }
    }
    if !root.transition_started && &anim.get_key() == "static" {
        root.transition_started = true;
        screen_effects.queue_effect(crate::drawing::effects::ScreenEffect::FadeToBlack(Some(
            GameState {
                meta: MetaState::Menu(MenuState::Title),
                pause: None,
            },
        )));
        screen_effects.queue_effect(crate::drawing::effects::ScreenEffect::UnfadeToBlack)
    }
}

fn destroy_studio_screen(
    mut commands: Commands,
    markers: Query<Entity, With<StudioScreenRoot>>,
    mut music_manager: ResMut<MusicManager>,
) {
    for eid in markers.iter() {
        commands.entity(eid).despawn_recursive();
    }
    // Spawn settings button and start menu music
    music_manager.fade_to_song(Some(MusicKind::EyeOfTheStorm));
    commands.spawn((
        MenuButtonBundle::new(
            MenuButton::basic("go_settings", "S"),
            GameRelativePlacement::new(IVec3::new(-151, -78, 0), 1.0),
        ),
        MenuSettingsButton,
    ));
}

fn is_in_studio_screen_helper(gs: &GameState) -> bool {
    match gs.meta {
        MetaState::Menu(menu_state) => match menu_state {
            MenuState::Studio => true,
            _ => false,
        },
        _ => false,
    }
}
fn is_in_studio_screen(gs: Res<GameState>) -> bool {
    is_in_studio_screen_helper(&gs)
}
when_becomes_true!(is_in_studio_screen_helper, entered_studio_screen);
when_becomes_false!(is_in_studio_screen_helper, left_studio_screen);

pub fn register_studio_screen(app: &mut App) {
    app.add_systems(Update, setup_studio_screen.run_if(entered_studio_screen));
    app.add_systems(Update, destroy_studio_screen.run_if(left_studio_screen));
    app.add_systems(
        Update,
        update_studio_screen
            .run_if(is_in_studio_screen)
            .after(update_any_menu),
    );
}

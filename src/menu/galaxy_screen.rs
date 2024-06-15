use bevy::prelude::*;

use crate::{
    drawing::{
        animation::{AnimationManager, MultiAnimationManager, SpriteInfo},
        layering::light_layer_u8,
        text::{TextAlign, TextBoxBundle, TextManager, TextNode, TextWeight},
    },
    meta::{
        game_state::{GameState, MenuState, MetaState},
        progress::{ActiveSaveFile, GalaxyKind, GameProgress},
    },
    physics::dyno::IntMoveableBundle,
    when_becomes_false, when_becomes_true,
};

/// Root of the galaxy screen. Destroyed on on_destroy
#[derive(Component)]
struct GalaxyScreenRoot;

#[derive(Component)]
struct GalaxyChoice {
    kind: GalaxyKind,
    selected: bool,
}

fn galaxy_offset(kind: GalaxyKind) -> i32 {
    kind.rank() as i32 * 72
}

#[derive(Bundle)]
struct GalaxyChoiceBundle {
    info: GalaxyChoice,
    multi: MultiAnimationManager,
    name: Name,
    text: TextManager,
    spatial: SpatialBundle,
}
impl GalaxyChoiceBundle {
    fn from_kind(kind: GalaxyKind) -> Self {
        let multi = match kind {
            GalaxyKind::Basic => MultiAnimationManager::from_pairs(vec![
                (
                    "galaxy",
                    AnimationManager::single_static(SpriteInfo {
                        path: "sprites/menu/galaxy/basic.png".to_string(),
                        size: UVec2::new(36, 36),
                        ..default()
                    }),
                ),
                (
                    "light",
                    AnimationManager::single_static(SpriteInfo {
                        path: "sprites/menu/galaxy/basicL.png".to_string(),
                        size: UVec2::new(36, 36),
                        ..default()
                    })
                    .force_render_layer(light_layer_u8()),
                ),
            ]),
            GalaxyKind::Springy => MultiAnimationManager::from_pairs(vec![
                (
                    "galaxy",
                    AnimationManager::single_static(SpriteInfo {
                        path: "sprites/menu/galaxy/basic.png".to_string(),
                        size: UVec2::new(36, 36),
                        ..default()
                    }),
                ),
                (
                    "light",
                    AnimationManager::single_static(SpriteInfo {
                        path: "sprites/menu/galaxy/basicL.png".to_string(),
                        size: UVec2::new(36, 36),
                        ..default()
                    })
                    .force_render_layer(light_layer_u8()),
                ),
            ]),
        };
        let meta = kind.to_meta_data();
        let text = TextManager::from_pairs(vec![(
            "title",
            TextNode {
                content: meta.title.clone(),
                size: 16.0,
                pos: IVec3::new(0, -24, 0),
                ..default()
            },
        )]);
        let spatial = SpatialBundle::from_transform(Transform::from_translation(Vec3::new(
            galaxy_offset(kind) as f32,
            0.0,
            0.0,
        )));
        // let mut text = TextBoxBundle::new_sprite_text(
        //     &meta.title,
        //     36.0,
        //     IVec3::new(0, -18, 0),
        //     Color::WHITE,
        //     TextWeight::Regular,
        //     TextAlign::Center,
        // );
        // text.inner.transform = Transform::from_translation(Vec3::new(x_offset, 0.0, 0.0));
        Self {
            info: GalaxyChoice {
                kind,
                selected: false,
            },
            multi,
            name: Name::new(format!("galaxy_choice_{}", kind)),
            text,
            spatial,
        }
    }
}

fn setup_galaxy_screen(
    mut commands: Commands,
    progress: Query<&GameProgress, With<ActiveSaveFile>>,
) {
    let progress = progress.single();
    let active_galaxy = progress.first_incomplete_galaxy();
    commands
        .spawn((
            GalaxyScreenRoot,
            Name::new("galaxy_screen"),
            IntMoveableBundle::new(IVec3::new(-galaxy_offset(active_galaxy), 0, 0)),
        ))
        .with_children(|parent| {
            for kind in GalaxyKind::all() {
                parent.spawn(GalaxyChoiceBundle::from_kind(kind));
            }
        });
}

fn update_galaxy_screen() {}

fn destroy_galaxy_screen(mut commands: Commands, root: Query<Entity, With<GalaxyScreenRoot>>) {
    let Ok(root) = root.get_single() else {
        error!("weird stuff in destroy_galaxy_screen");
        return;
    };
    commands.entity(root).despawn_recursive();
}

fn is_in_galaxy_screen_helper(gs: &GameState) -> bool {
    match gs.meta {
        MetaState::Menu(menu_state) => match menu_state {
            MenuState::GalaxyOverworld => true,
            _ => false,
        },
        _ => false,
    }
}
fn is_in_galaxy_screen(gs: Res<GameState>) -> bool {
    is_in_galaxy_screen_helper(&gs)
}
when_becomes_true!(is_in_galaxy_screen_helper, entered_galaxy_screen);
when_becomes_false!(is_in_galaxy_screen_helper, left_galaxy_screen);

pub fn register_galaxy_screen(app: &mut App) {
    app.add_systems(Update, setup_galaxy_screen.run_if(entered_galaxy_screen));
    app.add_systems(Update, destroy_galaxy_screen.run_if(left_galaxy_screen));
    app.add_systems(
        Update,
        update_galaxy_screen
            .run_if(is_in_galaxy_screen)
            .after(setup_galaxy_screen),
    );
}

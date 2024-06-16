use bevy::prelude::*;

use crate::{
    drawing::{
        animation::{AnimationManager, MultiAnimationManager, SpriteInfo},
        effects::EffectVal,
        layering::light_layer_u8,
        text::{TextManager, TextNode},
    },
    math::Spleen,
    meta::{
        game_state::{GameState, MenuState, MetaState},
        progress::{ActiveSaveFile, GalaxyKind, GameProgress},
    },
    physics::dyno::IntMoveableBundle,
    when_becomes_false, when_becomes_true,
};

/// Root of the galaxy screen. Destroyed on on_destroy
#[derive(Component)]
struct GalaxyScreenRoot {
    selected: GalaxyKind,
}

#[derive(Component, Default)]
struct GalaxyChoice {
    kind: GalaxyKind,
    shrink_helper: Option<f32>,
    grow_helper: Option<f32>,
}

#[derive(Component)]
struct LittleShip;

#[derive(Bundle)]
struct LittleShipBundle {
    little_ship: LittleShip,
    multi: MultiAnimationManager,
    spatial: SpatialBundle,
    name: Name,
}
impl LittleShipBundle {
    pub fn new(kind: GalaxyKind) -> Self {
        let multi = MultiAnimationManager::from_pairs(vec![
            (
                "ship",
                AnimationManager::single_static(SpriteInfo {
                    path: "sprites/ship.png".to_string(),
                    size: UVec2::new(8, 8),
                    ..default()
                }),
            ),
            (
                "light",
                AnimationManager::single_static(SpriteInfo {
                    path: "sprites/shipL.png".to_string(),
                    size: UVec2::new(64, 64),
                    ..default()
                })
                .force_render_layer(light_layer_u8()),
            ),
        ]);
        let x_offset = galaxy_offset(kind) as f32;
        let spatial = SpatialBundle::from_transform(Transform {
            translation: Vec3::new(x_offset, 0.0, 1.0),
            scale: Vec3::new(0.0, 0.0, 1.0),
            ..default()
        });
        Self {
            little_ship: LittleShip,
            multi,
            spatial,
            name: Name::new("little_ship"),
        }
    }
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
        Self {
            info: GalaxyChoice { kind, ..default() },
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
            GalaxyScreenRoot {
                selected: active_galaxy,
            },
            Name::new("galaxy_screen"),
            IntMoveableBundle::new(IVec3::new(-galaxy_offset(active_galaxy), 0, 0)),
        ))
        .with_children(|parent| {
            for kind in GalaxyKind::all() {
                parent.spawn(GalaxyChoiceBundle::from_kind(kind));
            }
            parent.spawn(LittleShipBundle::new(active_galaxy));
        });
}

fn handle_galaxy_screen_input(
    mut root: Query<(Entity, &mut GalaxyScreenRoot, &Transform)>,
    little_ship: Query<(Entity, &Transform), With<LittleShip>>,
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let Ok((eid, mut root, tran)) = root.get_single_mut() else {
        return;
    };
    let Ok((ship_eid, ship_tran)) = little_ship.get_single() else {
        return;
    };
    // First check if the user selected the galaxy by hitting enter
    if keyboard.just_pressed(KeyCode::Enter) {
        warn!("TODO: GalaxyOverworld enter press");
        return;
    }
    let new_kind = {
        if keyboard.just_pressed(KeyCode::ArrowLeft) {
            root.selected.prev()
        } else if keyboard.just_pressed(KeyCode::ArrowRight) {
            root.selected.next()
        } else {
            None
        }
    };
    let Some(new_kind) = new_kind else {
        // Nothing to do
        return;
    };
    let duration = 0.75;
    // Insert the root effect
    let start_val = tran.translation.x;
    let end_val = -galaxy_offset(new_kind) as f32;
    root.selected = new_kind;
    let effect_val = EffectVal::new(start_val, end_val, Spleen::EaseInOutCubic, duration);
    commands.entity(eid).insert(effect_val);
    // Insert the ship effect
    let start_val = ship_tran.scale.x;
    let end_val = 0.0;
    let effect_val = EffectVal::new(start_val, end_val, Spleen::EaseInOutCubic, duration);
    commands.entity(ship_eid).insert(effect_val);
}

fn update_galaxy_sizes(
    root: Query<(&GalaxyScreenRoot, Option<&EffectVal>)>,
    mut galaxys: Query<(&mut GalaxyChoice, &mut Transform)>,
) {
    let Ok((root, root_val)) = root.get_single() else {
        return;
    };
    for (mut choice, mut tran) in galaxys.iter_mut() {
        if choice.kind == root.selected {
            choice.shrink_helper = None;
            tran.scale = match root_val {
                Some(effect_val) => {
                    let watermark = match choice.grow_helper {
                        Some(old_scale) => old_scale,
                        None => {
                            choice.grow_helper = Some(tran.scale.x);
                            tran.scale.x
                        }
                    };
                    let mult = watermark + effect_val.interp_time() * (2.0 - watermark);
                    (Vec2::ONE * mult).extend(1.0)
                }
                None => Vec3::ONE * 2.0,
            };
        } else {
            choice.grow_helper = None;
            tran.scale = match root_val {
                Some(effect_val) => {
                    let watermark = match choice.shrink_helper {
                        Some(old_scale) => old_scale,
                        None => {
                            choice.shrink_helper = Some(tran.scale.x);
                            tran.scale.x
                        }
                    };
                    let mult = watermark + effect_val.interp_time() * (1.0 - watermark);
                    (Vec2::ONE * mult).extend(1.0)
                }
                None => Vec3::ONE,
            };
        }
    }
}

fn update_root_and_ship(
    mut root: Query<(Entity, &mut Transform, &EffectVal), With<GalaxyScreenRoot>>,
    mut little_ship: Query<
        (Entity, &mut Transform, &EffectVal),
        (With<LittleShip>, Without<GalaxyScreenRoot>),
    >,
    mut commands: Commands,
) {
    let Ok((eid, mut tran, effect_val)) = root.get_single_mut() else {
        return;
    };
    let Ok((ship_eid, mut ship_tran, ship_effect_val)) = little_ship.get_single_mut() else {
        return;
    };
    tran.translation.x = effect_val.interp();
    if effect_val.finished() {
        commands.entity(eid).remove::<EffectVal>();
    }
    ship_tran.translation.x = -tran.translation.x;
    // TODO: Kind of hacky, maybe add a midpoint spleen, would require a refactor tho
    let time_frac = ship_effect_val.timer.fraction();
    let ship_spleen = Spleen::EaseInOutCubic;
    let scale = if time_frac < 0.5 {
        let spleen_frac = ship_spleen.interp(time_frac * 2.0);
        ship_effect_val.get_start_val() + spleen_frac * (1.0 - ship_effect_val.get_start_val())
    } else {
        let spleen_frac = ship_spleen.interp((time_frac - 0.5) * 2.0);
        1.0 - spleen_frac
    };
    ship_tran.translation.y = scale * 18.0;
    ship_tran.scale.x = scale;
    ship_tran.scale.y = scale;
    if effect_val.finished() {
        commands.entity(ship_eid).remove::<EffectVal>();
    }
}

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
        handle_galaxy_screen_input
            .run_if(is_in_galaxy_screen)
            .after(setup_galaxy_screen),
    );
    app.add_systems(
        FixedUpdate,
        (
            handle_galaxy_screen_input,
            update_galaxy_sizes,
            update_root_and_ship,
        )
            .chain()
            .run_if(is_in_galaxy_screen),
    );
}

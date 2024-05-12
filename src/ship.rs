use crate::cutscenes::is_not_in_cutscene;
use crate::drawing::animation::{AnimationManager, MultiAnimationManager, SpriteInfo};
use crate::drawing::layering::light_layer_u8;
use crate::environment::particle::{
    ParticleBody, ParticleBundle, ParticleColoring, ParticleOptions, ParticleSizing,
};
use crate::environment::rock::{Rock, RockKind};
use crate::input::LaunchEvent;
use crate::input::LongKeyPress;
use crate::math::Spleen;
use crate::meta::game_state::{GameState, MetaState, SetGameState};
use crate::meta::level_data::LevelRoot;
use crate::physics::dyno::{apply_fields, IntDyno};
use crate::physics::should_apply_physics;
use bevy::prelude::*;

#[derive(Component)]
pub struct Ship {
    pub can_shoot: bool,
}
impl Ship {
    pub const fn radius() -> f32 {
        4.0
    }
}

#[derive(Bundle)]
pub struct ShipBundle {
    pub ship: Ship,
    pub respawn_watcher: LongKeyPress,
    pub dyno: IntDyno,
    pub spatial: SpatialBundle,
    pub anim: MultiAnimationManager,
    pub name: Name,
}
impl ShipBundle {
    pub fn new(pos: IVec2) -> Self {
        let ship = AnimationManager::single_static(SpriteInfo {
            path: "sprites/ship.png".to_string(),
            size: UVec2::new(8, 8),
        });
        let mut light = AnimationManager::single_static(SpriteInfo {
            path: "sprites/shipL.png".to_string(),
            size: UVec2::new(64, 64),
        });
        light.set_render_layers(vec![light_layer_u8()]);
        Self {
            ship: Ship { can_shoot: false },
            respawn_watcher: LongKeyPress::new(KeyCode::KeyR, 45),
            dyno: IntDyno::new(pos.extend(10), 4.0),
            spatial: SpatialBundle::from_transform(Transform::from_translation(
                pos.as_vec2().extend(100.0),
            )),
            anim: MultiAnimationManager::from_pairs(vec![("ship", ship), ("light", light)]),
            name: Name::new("Ship"),
        }
    }
}

pub fn launch_ship(
    mut ship_q: Query<(&mut IntDyno, &mut Ship)>,
    mut launch_events: EventReader<LaunchEvent>,
    gs: Res<GameState>,
    mut gs_writer: EventWriter<SetGameState>,
) {
    let level_state = gs.get_level_state();
    for launch in launch_events.read() {
        for (mut dyno, mut ship) in ship_q.iter_mut() {
            if !ship.can_shoot && level_state.is_some() {
                continue;
            }
            dyno.vel = launch.vel;
            ship.can_shoot = false;
            if let Some(mut ls) = level_state.clone() {
                ls.num_shots += 1;
                gs_writer.send(SetGameState(GameState {
                    meta: MetaState::Level(ls.clone()),
                }));
            }
        }
    }
}

fn watch_for_respawn(
    mut commands: Commands,
    gs: Res<GameState>,
    mut entity_n_lp: Query<(Entity, &mut LongKeyPress), With<Ship>>,
    level_root_q: Query<Entity, With<LevelRoot>>,
) {
    let MetaState::Level(level_state) = &gs.meta else {
        return;
    };
    for (id, mut lp) in entity_n_lp.iter_mut() {
        if lp.was_activated() {
            commands.entity(id).despawn_recursive();
            let level_root_eid = level_root_q.single();
            commands.entity(level_root_eid).with_children(|parent| {
                parent.spawn(ShipBundle::new(level_state.last_safe_location));
            });
        }
    }
}

fn watch_for_death(
    mut commands: Commands,
    gs: Res<GameState>,
    entity_n_lp: Query<(Entity, &IntDyno), With<Ship>>,
    rock_info: Query<&Rock>,
    level_root_q: Query<Entity, With<LevelRoot>>,
) {
    let MetaState::Level(level_state) = &gs.meta else {
        return;
    };
    for (id, dyno) in entity_n_lp.iter() {
        for rock_id in dyno.statics.iter() {
            let Ok(rock) = rock_info.get(*rock_id) else {
                continue;
            };
            if rock.kind == RockKind::SimpleKill {
                commands.entity(id).despawn_recursive();
                let level_root_eid = level_root_q.single();
                commands.entity(level_root_eid).with_children(|parent| {
                    parent.spawn(ShipBundle::new(level_state.last_safe_location));
                });
                break;
            }
        }
    }
}

fn replenish_shot(
    mut ship_q: Query<(&mut Ship, &mut IntDyno, &GlobalTransform)>,
    gs: Res<GameState>,
    mut gs_writer: EventWriter<SetGameState>,
) {
    let Some(level_state) = gs.get_level_state() else {
        return;
    };
    for (mut ship, mut dyno, tran) in ship_q.iter_mut() {
        if ship.can_shoot {
            continue;
        }
        let ipos = IVec2 {
            x: tran.translation().x as i32,
            y: tran.translation().y as i32,
        };
        if dyno.vel.length() < 3.0 && dyno.statics.len() > 0 {
            ship.can_shoot = true;
            dyno.vel = Vec2::ZERO;
            let mut ls = level_state.clone();
            ls.last_safe_location = ipos;
            gs_writer.send(SetGameState(GameState {
                meta: MetaState::Level(ls),
            }));
        }
    }
}

pub fn spawn_trail(
    mut commands: Commands,
    ship: Query<&GlobalTransform, With<Ship>>,
    mut mats: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Ok(tran) = ship.get_single() else {
        return;
    };
    ParticleBundle::spawn_options(
        &mut commands,
        ParticleBody {
            pos: tran.translation().truncate(),
            vel: Vec2::ZERO,
            size: Ship::radius(),
            color: Color::YELLOW,
        },
        0.5,
        ParticleOptions {
            sizing: Some(ParticleSizing {
                spleen: Spleen::EaseInQuad,
            }),
            coloring: Some(ParticleColoring {
                end_color: Color::BLUE,
                spleen: Spleen::EaseInQuad,
            }),
            ..default()
        },
        &mut mats,
        &mut meshes,
    );
}

pub fn register_ship(app: &mut App) {
    app.add_systems(
        Update,
        launch_ship
            .run_if(should_apply_physics)
            .run_if(is_not_in_cutscene),
    );
    app.add_systems(
        Update,
        (
            watch_for_respawn,
            replenish_shot,
            watch_for_death,
            spawn_trail,
        )
            .run_if(should_apply_physics)
            .run_if(is_not_in_cutscene)
            .after(apply_fields),
    );
}

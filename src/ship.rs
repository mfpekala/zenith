use std::f32::consts::PI;

use crate::cutscenes::is_not_in_cutscene;
use crate::drawing::animation::{
    AnimationManager, AnimationNode, MultiAnimationManager, SpriteInfo,
};
use crate::drawing::layering::light_layer_u8;
use crate::environment::live_poly::LivePolyMarker;
use crate::environment::particle::{
    ParticleBody, ParticleBundle, ParticleColoring, ParticleOptions, ParticleSizing, ParticleVel,
};
use crate::environment::replenish::{ReplenishCharging, ReplenishMarker};
use crate::environment::rock::{Rock, RockKind};
use crate::input::LaunchEvent;
use crate::input::LongKeyPress;
use crate::leveler::load::destroy_level;
use crate::math::Spleen;
use crate::meta::consts::FRAMERATE;
use crate::meta::game_state::{GameState, MetaState, SetMetaState};
use crate::meta::level_data::LevelRoot;
use crate::physics::collider::ColliderActive;
use crate::physics::dyno::{apply_fields, IntDyno};
use crate::physics::{should_apply_physics, BulletTime};
use crate::sound::effect::SoundEffect;
use bevy::prelude::*;
use rand::{thread_rng, Rng};

#[derive(Component)]
pub struct Ship {
    pub can_shoot: bool,
    pub last_safe_location: IVec2,
    pub time_in_goal: f32,
    pub dist_to_goal_center_sq: f32,
    pub finished: bool,
}
impl Ship {
    pub const fn radius() -> f32 {
        4.0
    }
}

#[derive(Component)]
pub enum Dead {
    Explosion,
}

#[derive(Component)]
struct Dying {
    timer: Timer,
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
        let ship = AnimationManager::from_static_pairs(vec![
            (
                "full",
                SpriteInfo {
                    path: "sprites/ship.png".to_string(),
                    size: UVec2::new(8, 8),
                    ..default()
                },
            ),
            (
                "empty",
                SpriteInfo {
                    path: "sprites/ship_empty.png".to_string(),
                    size: UVec2::new(8, 8),
                    ..default()
                },
            ),
        ]);
        let light = AnimationManager::from_nodes(vec![
            (
                "full",
                AnimationNode {
                    length: 1,
                    sprite: SpriteInfo {
                        path: "sprites/shipL.png".to_string(),
                        size: UVec2::new(64, 64),
                        ..default()
                    },
                    ..default()
                },
            ),
            (
                "exploding",
                AnimationNode {
                    length: 4,
                    sprite: SpriteInfo {
                        path: "sprites/shipL_explosion.png".to_string(),
                        size: UVec2::new(64, 64),
                        ..default()
                    },
                    next: Some("gone".into()),
                    ..default()
                },
            ),
            (
                "gone",
                AnimationNode {
                    length: 1,
                    sprite: SpriteInfo {
                        path: "sprites/shipL_empty.png".to_string(),
                        size: UVec2::new(64, 64),
                        ..default()
                    },
                    ..default()
                },
            ),
        ])
        .force_render_layer(light_layer_u8());
        Self {
            ship: Ship {
                can_shoot: true,
                last_safe_location: pos,
                time_in_goal: 0.0,
                dist_to_goal_center_sq: f32::MAX,
                finished: false,
            },
            respawn_watcher: LongKeyPress::new(KeyCode::KeyR, (FRAMERATE * 0.36) as u32),
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
    mut gs_writer: EventWriter<SetMetaState>,
    bullet_time: Res<BulletTime>,
    mut commands: Commands,
) {
    let level_state = gs.get_level_state();
    for launch in launch_events.read() {
        for (mut dyno, mut ship) in ship_q.iter_mut() {
            if !ship.can_shoot {
                continue;
            }
            dyno.vel = launch.vel * bullet_time.factor();
            ship.can_shoot = false;
            if let Some(mut ls) = level_state.clone() {
                ls.num_shots += 1;
                gs_writer.send(SetMetaState(MetaState::Level(ls.clone())));
            }
            commands.spawn(SoundEffect::universal(
                "sound_effects/shoot.ogg",
                0.2,
                false,
            ));
        }
    }
}

/// Checks if the user held down r to respawn
fn watch_for_respawn(mut commands: Commands, mut entity_n_lp: Query<(Entity, &mut LongKeyPress)>) {
    for (id, mut lp) in entity_n_lp.iter_mut() {
        if lp.was_activated() {
            commands.entity(id).insert(Dead::Explosion);
        }
    }
}

/// Checks if the ship hit a simple kill rock
fn watch_simple_kill_collisions(
    mut commands: Commands,
    ship_q: Query<(Entity, &IntDyno)>,
    rock_info: Query<&Rock>,
) {
    for (id, dyno) in ship_q.iter() {
        for rock_id in dyno.statics.keys() {
            let Ok(rock) = rock_info.get(*rock_id) else {
                continue;
            };
            if rock.kind == RockKind::SimpleKill {
                commands.entity(id).insert(Dead::Explosion);
                break;
            }
        }
    }
}

/// Checks if there are ready LivePoly's that the ship is not colliding with.
/// If so we assume we're "out of bounds" and kill the ship
fn watch_oob(
    mut commands: Commands,
    ship_q: Query<(Entity, &IntDyno), (With<Ship>, Without<Dead>)>,
    live_polys: Query<(Entity, &LivePolyMarker)>,
) {
    for (eid, dyno) in ship_q.iter() {
        let mut oob = true;
        for (lpid, lp) in live_polys.iter() {
            if !lp.ready || dyno.triggers.contains_key(&lpid) {
                oob = false;
                break;
            }
        }
        if oob {
            commands.entity(eid).insert(Dead::Explosion);
        }
    }
}

/// Checks if the shot can be replenished and also updates the sprite
fn replenish_shot(
    mut ship_q: Query<
        (&mut Ship, &mut IntDyno, &mut MultiAnimationManager),
        Without<ReplenishMarker>,
    >,
    mut replenishes: Query<
        (Entity, &mut MultiAnimationManager),
        (With<ReplenishMarker>, With<ColliderActive>),
    >,
    mut commands: Commands,
    bullet_time: Res<BulletTime>,
) {
    for (mut ship, mut dyno, mut multi) in ship_q.iter_mut() {
        let can_shoot_before = ship.can_shoot;

        if dyno.vel.length() < 0.0001 * bullet_time.factor() && dyno.statics.len() > 0 {
            ship.last_safe_location = dyno.ipos.truncate();
            ship.can_shoot = true;
        }
        if dyno.long_statics.iter().any(|(_key, val)| *val >= 3) {
            ship.last_safe_location = dyno.ipos.truncate();
            ship.can_shoot = true;
        }
        let mut replenish_triggers = vec![];
        for (trigger_id, _) in dyno.triggers.iter() {
            if !replenishes.contains(*trigger_id) {
                continue;
            }
            replenish_triggers.push(*trigger_id);
        }
        dyno.triggers
            .retain(|id, _| !replenish_triggers.contains(id));
        if !ship.can_shoot && replenish_triggers.len() > 0 {
            ship.last_safe_location = dyno.ipos.truncate();
            ship.can_shoot = true;
            for eid in replenish_triggers {
                let (rid, mut repl) = replenishes.get_mut(eid).unwrap();
                let core = repl.map.get_mut("core").unwrap();
                core.set_key("exploding");
                let light = repl.map.get_mut("light").unwrap();
                light.set_key("exploding");
                commands.entity(rid).remove::<ColliderActive>();
                commands.entity(rid).insert(ReplenishCharging::new());
            }
        }
        let anim = multi.map.get_mut("ship").unwrap();
        let key = if ship.can_shoot { "full" } else { "empty" };
        anim.set_key(key);

        // Only if we mark can shoot on this frame do we play a sound effect
        if !can_shoot_before && ship.can_shoot {
            commands.spawn(SoundEffect::universal(
                "sound_effects/recharge.ogg",
                0.16,
                false,
            ));
        }
    }
}

/// Spawns a fun trail behind the ball
pub fn spawn_trail(
    mut commands: Commands,
    ship: Query<&GlobalTransform, (With<Ship>, Without<Dead>)>,
    level_root: Query<Entity, With<LevelRoot>>,
) {
    let Ok(tran) = ship.get_single() else {
        return;
    };
    // TODO: I should attach a particle spawner to the ship, and then have the particle spawner
    // handle doing this
    let Ok(level_root) = level_root.get_single() else {
        return;
    };
    let id = ParticleBundle::spawn_options(
        &mut commands,
        ParticleBody {
            pos: tran.translation() - Vec3::Z,
            vel: Vec2::ZERO,
            size: Ship::radius(),
            color: Color::YELLOW,
            ..default()
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
    );
    commands.entity(level_root).add_child(id);
}

/// Checks to see if the ship has been marked for death. If so, start the death effect.
fn watch_for_dead_ships(
    mut ships: Query<
        (
            Entity,
            &Dead,
            &mut MultiAnimationManager,
            &IntDyno,
            &GlobalTransform,
        ),
        (With<Ship>, Without<Dying>),
    >,
    mut commands: Commands,
) {
    let mut rng = thread_rng();
    for (eid, cause, mut anim, dyno, gtran) in ships.iter_mut() {
        match cause {
            Dead::Explosion => {
                commands.spawn(SoundEffect::spatial(
                    "sound_effects/explosion.ogg",
                    1.0,
                    false,
                ));
                commands.entity(eid).insert(Dying {
                    timer: Timer::from_seconds(0.2, TimerMode::Once),
                });
                anim.map.get_mut("ship").unwrap().set_hidden(true);
                anim.map.get_mut("light").unwrap().set_key("exploding");
                for _ in (0..16).into_iter() {
                    let angle = rng.gen::<f32>() * 2.0 * PI;
                    let added_vel = Vec2::from_angle(angle);
                    let start_vel = dyno.vel / 10.0 + added_vel / 5.0;
                    // Sprite layer particle
                    ParticleBundle::spawn_options(
                        &mut commands,
                        ParticleBody {
                            pos: gtran.translation() - Vec3::Z,
                            vel: start_vel,
                            size: Ship::radius(),
                            color: Color::WHITE,
                            ..default()
                        },
                        0.5,
                        ParticleOptions {
                            sizing: Some(ParticleSizing {
                                spleen: Spleen::EaseInQuad,
                            }),
                            coloring: Some(ParticleColoring {
                                end_color: Color::RED,
                                spleen: Spleen::EaseInQuad,
                            }),
                            vel: Some(ParticleVel {
                                start_vel,
                                end_vel: Vec2::ZERO,
                                spleen: Spleen::EaseInQuad,
                            }),
                        },
                    );
                    // Light layer particle
                    ParticleBundle::spawn_options(
                        &mut commands,
                        ParticleBody {
                            pos: gtran.translation() - Vec3::Z,
                            vel: start_vel,
                            size: Ship::radius() * 3.0,
                            color: Color::WHITE,
                            layer: light_layer_u8(),
                        },
                        0.5,
                        ParticleOptions {
                            sizing: Some(ParticleSizing {
                                spleen: Spleen::EaseInQuad,
                            }),
                            coloring: Some(ParticleColoring {
                                end_color: Color::RED,
                                spleen: Spleen::EaseInQuad,
                            }),
                            vel: Some(ParticleVel {
                                start_vel,
                                end_vel: Vec2::ZERO,
                                spleen: Spleen::EaseInQuad,
                            }),
                        },
                    );
                }
            }
        }
    }
}

/// Updates dying ships, eventually despawning and respawning them
fn update_dying_ships(
    mut ships: Query<(Entity, &Ship, &mut IntDyno, &mut Dying)>,
    mut commands: Commands,
    time: Res<Time>,
    level_root_q: Query<Entity, With<LevelRoot>>,
) {
    let Ok(level_root_eid) = level_root_q.get_single() else {
        return;
    };
    for (eid, ship, mut dyno, mut dying) in ships.iter_mut() {
        dying.timer.tick(time.delta());
        dyno.vel *= 0.4;
        if dying.timer.finished() {
            commands.entity(eid).despawn_recursive();
            commands.entity(level_root_eid).with_children(|parent| {
                parent.spawn(ShipBundle::new(ship.last_safe_location));
            });
        }
    }
}

pub fn register_ship(app: &mut App) {
    app.add_systems(
        Update,
        launch_ship
            .run_if(should_apply_physics)
            .run_if(is_not_in_cutscene),
    );
    app.add_systems(
        FixedUpdate,
        (
            replenish_shot,
            watch_simple_kill_collisions,
            watch_oob,
            watch_for_dead_ships,
            update_dying_ships,
        )
            .run_if(should_apply_physics)
            .run_if(is_not_in_cutscene)
            .after(apply_fields),
    );

    app.add_systems(
        Update,
        (watch_for_respawn, spawn_trail)
            .run_if(should_apply_physics)
            .run_if(is_not_in_cutscene)
            .after(apply_fields)
            .after(destroy_level),
    );
}

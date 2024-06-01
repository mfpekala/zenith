use crate::cutscenes::is_not_in_cutscene;
use crate::drawing::animation::{AnimationManager, MultiAnimationManager, SpriteInfo};
use crate::drawing::layering::light_layer_u8;
use crate::environment::particle::{
    ParticleBody, ParticleBundle, ParticleColoring, ParticleOptions, ParticleSizing,
};
use crate::environment::replenish::{ReplenishCharging, ReplenishMarker};
use crate::environment::rock::{Rock, RockKind};
use crate::input::LaunchEvent;
use crate::input::LongKeyPress;
use crate::math::Spleen;
use crate::meta::game_state::{GameState, MetaState, SetGameState};
use crate::meta::level_data::LevelRoot;
use crate::physics::collider::ColliderActive;
use crate::physics::dyno::{apply_fields, IntDyno};
use crate::physics::{should_apply_physics, BulletTime};
use bevy::prelude::*;

#[derive(Component)]
pub struct Ship {
    pub can_shoot: bool,
    pub last_safe_location: IVec2,
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
        let ship = AnimationManager::from_static_pairs(vec![
            (
                "full",
                SpriteInfo {
                    path: "sprites/ship.png".to_string(),
                    size: UVec2::new(8, 8),
                },
            ),
            (
                "empty",
                SpriteInfo {
                    path: "sprites/ship_empty.png".to_string(),
                    size: UVec2::new(8, 8),
                },
            ),
        ]);
        let mut light = AnimationManager::single_static(SpriteInfo {
            path: "sprites/shipL.png".to_string(),
            size: UVec2::new(64, 64),
        });
        light.set_render_layers(vec![light_layer_u8()]);
        Self {
            ship: Ship {
                can_shoot: true,
                last_safe_location: pos,
            },
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
    bullet_time: Res<BulletTime>,
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
                gs_writer.send(SetGameState(GameState {
                    meta: MetaState::Level(ls.clone()),
                }));
            }
        }
    }
}

fn watch_for_respawn(
    mut commands: Commands,
    mut entity_n_lp: Query<(Entity, &mut LongKeyPress, &Ship)>,
    level_root_q: Query<Entity, With<LevelRoot>>,
) {
    for (id, mut lp, ship) in entity_n_lp.iter_mut() {
        if lp.was_activated() {
            commands.entity(id).despawn_recursive();
            let level_root_eid = level_root_q.single();
            commands.entity(level_root_eid).with_children(|parent| {
                parent.spawn(ShipBundle::new(ship.last_safe_location));
            });
        }
    }
}

fn watch_for_death(
    mut commands: Commands,
    ship_q: Query<(Entity, &IntDyno, &Ship)>,
    rock_info: Query<&Rock>,
    level_root_q: Query<Entity, With<LevelRoot>>,
) {
    for (id, dyno, ship) in ship_q.iter() {
        for rock_id in dyno.statics.iter() {
            let Ok(rock) = rock_info.get(*rock_id) else {
                continue;
            };
            if rock.kind == RockKind::SimpleKill {
                commands.entity(id).despawn_recursive();
                let level_root_eid = level_root_q.single();
                commands.entity(level_root_eid).with_children(|parent| {
                    parent.spawn(ShipBundle::new(ship.last_safe_location));
                });
                break;
            }
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
        (Entity, &mut MultiAnimationManager, &mut ColliderActive),
        With<ReplenishMarker>,
    >,
    mut commands: Commands,
    bullet_time: Res<BulletTime>,
) {
    for (mut ship, mut dyno, mut multi) in ship_q.iter_mut() {
        if dyno.vel.length() < 0.0001 * bullet_time.factor() && dyno.statics.len() > 0 {
            ship.can_shoot = true;
        }
        if dyno.long_statics.iter().any(|(_key, val)| *val >= 3) {
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
            ship.can_shoot = true;
            for eid in replenish_triggers {
                let (rid, mut repl, mut active) = replenishes.get_mut(eid).unwrap();
                let core = repl.map.get_mut("core").unwrap();
                core.set_key("exploding");
                let light = repl.map.get_mut("light").unwrap();
                light.set_key("exploding");
                active.0 = false;
                commands.entity(rid).insert(ReplenishCharging::new());
            }
        }
        let anim = multi.map.get_mut("ship").unwrap();
        let key = if ship.can_shoot { "full" } else { "empty" };
        anim.set_key(key);
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
        FixedUpdate,
        (replenish_shot)
            .run_if(should_apply_physics)
            .run_if(is_not_in_cutscene)
            .after(apply_fields),
    );

    app.add_systems(
        Update,
        (watch_for_respawn, watch_for_death, spawn_trail)
            .run_if(should_apply_physics)
            .run_if(is_not_in_cutscene)
            .after(apply_fields),
    );
}

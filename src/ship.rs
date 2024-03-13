use crate::cutscenes::is_not_in_cutscene;
use crate::drawing::layering::sprite_layer;
use crate::drawing::light::RegularLightBundle;
use crate::drawing::mesh::generate_new_color_mesh;
use crate::environment::particle::{
    ParticleBody, ParticleBundle, ParticleColoring, ParticleOptions, ParticleSizing,
};
use crate::environment::rock::{Rock, RockKind};
use crate::input::LaunchEvent;
use crate::input::LongKeyPress;
use crate::math::{regular_polygon, Spleen};
use crate::meta::game_state::{GameState, MetaState, SetGameState};
use crate::physics::dyno::{resolve_dynos, IntDyno};
use crate::physics::should_apply_physics;
use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::sprite::MaterialMesh2dBundle;

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
    pub launch_preview: LaunchPreview,
    pub mesh: MaterialMesh2dBundle<ColorMaterial>,
    pub render_layers: RenderLayers,
}

#[derive(Resource)]
pub struct SpawnShipId(pub SystemId<(bevy::prelude::IVec2, f32)>);
pub fn spawn_ship(
    In((pos, radius)): In<(IVec2, f32)>,
    mut mats: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let mat = mats.add(ColorMaterial::from(Color::Hsla {
        hue: 0.0,
        saturation: 1.0,
        lightness: 1.0,
        alpha: 1.0,
    }));
    let points = regular_polygon(6, 45.0, radius);
    let mut mesh = generate_new_color_mesh(&points, &mat, &mut meshes);
    mesh.transform.translation = pos.as_vec2().extend(1.0);
    commands
        .spawn(ShipBundle {
            ship: Ship { can_shoot: false },
            respawn_watcher: LongKeyPress::new(KeyCode::KeyR, 45),
            dyno: IntDyno {
                vel: Vec2::ZERO,
                pos,
                rem: Vec2::ZERO,
                radius: 4.0,
                statics: vec![],
                triggers: vec![],
            },
            launch_preview: LaunchPreview::new(),
            mesh,
            render_layers: sprite_layer(),
        })
        .with_children(|parent| {
            parent.spawn(RegularLightBundle::new(12, 60.0, &mut mats, &mut meshes));
        });
}

#[derive(Component)]
pub struct LaunchPreview {
    pub tick: u32,
    pub speed: u32,
    pub num_skins: u32,
    pub ticks_between_skins: u32,
}
impl LaunchPreview {
    pub fn new() -> Self {
        Self {
            tick: 0,
            speed: 3,
            num_skins: 12,
            ticks_between_skins: 12,
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
    spawn_ship_id: Res<SpawnShipId>,
) {
    let MetaState::Level(level_state) = &gs.meta else {
        return;
    };
    for (id, mut lp) in entity_n_lp.iter_mut() {
        if lp.was_activated() {
            commands.entity(id).despawn_recursive();
            commands.run_system_with_input(
                spawn_ship_id.0,
                (level_state.last_safe_location, Ship::radius()),
            );
        }
    }
}

fn watch_for_death(
    mut commands: Commands,
    gs: Res<GameState>,
    entity_n_lp: Query<(Entity, &IntDyno), With<Ship>>,
    spawn_ship_id: Res<SpawnShipId>,
    rock_info: Query<&Rock>,
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
                commands.run_system_with_input(
                    spawn_ship_id.0,
                    (level_state.last_safe_location, Ship::radius()),
                );
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
    let spawn_id = app.world.register_system(spawn_ship);
    app.insert_resource(SpawnShipId(spawn_id));
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
            .after(resolve_dynos),
    );
}

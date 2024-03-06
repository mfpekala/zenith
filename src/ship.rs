use crate::drawing::hollow::HollowDrawable;
use crate::drawing::mesh::generate_new_mesh;
use crate::environment::goal::Goal;
use crate::environment::rock::RockKind;
use crate::environment::{field::Field, rock::Rock};
use crate::input::{LongKeyPress, MouseState};
use crate::math::regular_polygon;
use crate::meta::game_state::{in_editor, in_level, GameState, MetaState, SetGameState};
use crate::physics::{
    gravity_helper, move_dyno_helper, move_dynos, should_apply_physics, AvgDeltaTime,
};
use crate::{input::LaunchEvent, physics::Dyno};
use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

#[derive(Component)]
pub struct Ship {
    pub can_shoot: bool,
}

#[derive(Bundle)]
pub struct ShipBundle {
    pub ship: Ship,
    pub respawn_watcher: LongKeyPress,
    pub dyno: Dyno,
    pub launch_preview: LaunchPreview,
    pub mesh: MaterialMesh2dBundle<ColorMaterial>,
}

#[derive(Resource)]
pub struct SpawnShipId(pub SystemId<(bevy::prelude::Vec2, f32)>);
pub fn spawn_ship(
    In((pos, radius)): In<(Vec2, f32)>,
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
    let points = regular_polygon(12, 0.0, radius);
    let mut mesh = generate_new_mesh(&points, &mat, &mut meshes);
    mesh.transform.translation = pos.extend(0.0);
    commands.spawn(ShipBundle {
        ship: Ship { can_shoot: false },
        respawn_watcher: LongKeyPress::new(KeyCode::KeyR, 45),
        dyno: Dyno {
            vel: Vec2::ZERO,
            radius,
            touching_rock: None,
        },
        launch_preview: LaunchPreview::new(),
        mesh,
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

fn draw_ships(dyno_n_trans: Query<(&Dyno, &Transform)>, mut gz: Gizmos) {
    for (dyno, tran) in dyno_n_trans.iter() {
        dyno.draw_hollow(tran.translation.truncate(), &mut gz);
    }
}

fn draw_launch_previews(
    mut ship_q: Query<(&mut LaunchPreview, &Ship, &Dyno, &Transform)>,
    mouse_state: Res<MouseState>,
    mut gz: Gizmos,
    rocks: Query<(&Rock, &Transform), Without<Dyno>>,
    fields: Query<(&Field, &GlobalTransform), Without<Dyno>>,
    goal: Query<(&Goal, &Transform)>,
    gs: Res<GameState>,
    avg_time: Res<AvgDeltaTime>,
) {
    let Some(launch_vel) = mouse_state.pending_launch_vel else {
        return;
    };
    for (mut prev, ship, dyno, tran) in ship_q.iter_mut() {
        if !ship.can_shoot && !gs.is_in_editor() {
            continue;
        }
        let mut scratch_dyno = dyno.clone();
        scratch_dyno.vel = launch_vel;
        let mut scratch_point = tran.translation.truncate();
        // Offset
        let prev_applied = prev.tick / prev.speed;
        for _tick in 0..prev_applied {
            gravity_helper(
                &mut scratch_dyno,
                &scratch_point,
                &fields,
                &goal,
                avg_time.get_avg(),
            );
            move_dyno_helper(
                &mut scratch_dyno,
                &mut scratch_point,
                &rocks,
                avg_time.get_avg(),
            );
        }
        prev.tick = (prev.tick + 1) % (prev.ticks_between_skins * prev.speed);
        // Draw the damn things
        for skin in 0..prev.num_skins {
            let alpha = 1.0
                - (prev_applied as f32 + skin as f32 * prev.ticks_between_skins as f32)
                    / (prev.num_skins as f32 * prev.ticks_between_skins as f32);
            gz.circle_2d(scratch_point, 5.0, Color::rgba(0.7, 0.7, 0.7, alpha));
            for _ in 0..prev.ticks_between_skins {
                gravity_helper(
                    &mut scratch_dyno,
                    &scratch_point,
                    &fields,
                    &goal,
                    avg_time.get_avg(),
                );
                move_dyno_helper(
                    &mut scratch_dyno,
                    &mut scratch_point,
                    &rocks,
                    avg_time.get_avg(),
                );
            }
        }
    }
}

fn launch_ship(
    mut ship_q: Query<(&mut Dyno, &mut Ship)>,
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
            commands.run_system_with_input(spawn_ship_id.0, (level_state.last_safe_location, 16.0));
        }
    }
}

fn watch_for_death(
    mut commands: Commands,
    gs: Res<GameState>,
    entity_n_lp: Query<(Entity, &Dyno), With<Ship>>,
    spawn_ship_id: Res<SpawnShipId>,
) {
    let MetaState::Level(level_state) = &gs.meta else {
        return;
    };
    for (id, dyno) in entity_n_lp.iter() {
        if dyno.touching_rock == Some(RockKind::SimpleKill) {
            commands.entity(id).despawn_recursive();
            commands.run_system_with_input(spawn_ship_id.0, (level_state.last_safe_location, 16.0));
        }
    }
}

fn replenish_shot(
    mut ship_q: Query<(&mut Ship, &mut Dyno, &GlobalTransform)>,
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
        if dyno.vel.length() < 1.0
            && dyno.touching_rock.is_some()
            && dyno.touching_rock != Some(RockKind::SimpleKill)
        {
            ship.can_shoot = true;
            dyno.vel = Vec2::ZERO;
            let mut ls = level_state.clone();
            ls.last_safe_location = tran.translation().truncate();
            gs_writer.send(SetGameState(GameState {
                meta: MetaState::Level(ls),
            }));
        }
    }
}

pub fn register_ship(app: &mut App) {
    let spawn_id = app.world.register_system(spawn_ship);
    app.insert_resource(SpawnShipId(spawn_id));
    app.add_systems(Update, launch_ship.run_if(should_apply_physics));
    app.add_systems(
        Update,
        draw_ships
            .after(move_dynos)
            .run_if(in_editor.or_else(in_level)),
    );
    app.add_systems(
        Update,
        (
            draw_launch_previews,
            watch_for_respawn,
            replenish_shot,
            watch_for_death,
        )
            .after(draw_ships)
            .run_if(should_apply_physics),
    );
}

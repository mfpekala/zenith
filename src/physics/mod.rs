use bevy::prelude::*;

pub mod collider;
pub mod dyno;

use crate::{
    environment::{
        field::Field,
        goal::Goal,
        rock::{Rock, RockKind},
    },
    meta::game_state::{EditorState, GameState, MetaState},
};

use self::{collider::register_colliders, dyno::register_int_dynos};

#[derive(Resource)]
pub struct AvgDeltaTime {
    pub data: Vec<f32>,
}
impl AvgDeltaTime {
    pub fn new() -> Self {
        AvgDeltaTime { data: vec![] }
    }

    pub fn update(&mut self, delta: f32) {
        self.data.insert(0, delta);
        if self.data.len() > 50 {
            self.data.pop();
        }
    }

    pub fn get_avg(&self) -> f32 {
        if self.data.len() == 0 {
            return 0.01;
        }
        let mut sum = 0.0;
        for delta in self.data.iter() {
            sum += delta;
        }
        sum / (self.data.len() as f32)
    }
}

#[derive(Component, Clone, Debug)]
pub struct Dyno {
    pub vel: Vec2,
    pub radius: f32,
    pub touching_rock: Option<RockKind>,
}

pub fn resolve_dyno_rock_collisions(
    dyno: &mut Dyno,
    point: &mut Vec2,
    rocks: &Query<(&Rock, &Transform), Without<Dyno>>,
) -> Option<RockKind> {
    let radius = dyno.radius;
    let mut min_dist: Option<f32> = None;
    let mut min_point: Option<Vec2> = None;
    let mut min_rock: Option<&Rock> = None;
    for (rock, trans) in rocks.iter() {
        let closest_point = rock.closest_point(&point, &trans.translation.truncate());
        let dist = point.distance(closest_point);
        if min_dist.is_none() || dist < min_dist.unwrap() {
            min_dist = Some(dist);
            min_point = Some(closest_point);
            min_rock = Some(rock);
        }
    }
    if min_point.is_none() || min_dist.unwrap_or(f32::MAX) > radius {
        // No collision
        return None;
    }
    // First move the dyno out of the rock
    let min_point = min_point.unwrap();
    let diff = *point - min_point;
    *point += diff.normalize() * (radius - diff.length());
    // Then reflect the velocity
    let min_rock = min_rock.unwrap();
    let normal = diff.normalize();
    let pure_parr = -1.0 * dyno.vel.dot(normal) * normal + dyno.vel;
    let new_vel = pure_parr * (1.0 - min_rock.features.friction)
        - 1.0 * dyno.vel.dot(normal) * normal * min_rock.features.bounciness;
    dyno.vel = new_vel;
    Some(min_rock.kind.clone())
}

pub struct MoveDynoResult {
    pub touched_rock: Option<RockKind>,
}
impl MoveDynoResult {
    pub fn new() -> Self {
        Self { touched_rock: None }
    }
}

pub fn move_dyno_helper(
    dyno: &mut Dyno,
    point: &mut Vec2,
    rocks: &Query<(&Rock, &Transform), Without<Dyno>>,
    time_delta: f32,
) -> MoveDynoResult {
    let mut left_to_move = dyno.vel.length() * time_delta;
    let mut result = MoveDynoResult::new();
    while left_to_move > 0.0 {
        if dyno.vel.length() <= 0.000001 {
            // Prevent little movements
            dyno.vel = Vec2 { x: 0.0, y: 0.0 };
            break;
        }
        let moving_this_step = left_to_move.min(2.0);
        *point += dyno.vel.normalize() * moving_this_step;
        let this_step = resolve_dyno_rock_collisions(dyno, point, &rocks);
        if result.touched_rock.is_none() {
            result.touched_rock = this_step;
        }
        left_to_move -= moving_this_step;
    }
    result
}

pub fn move_dynos(
    mut dynos: Query<(&mut Dyno, &mut Transform), Without<Rock>>,
    rocks: Query<(&Rock, &Transform), Without<Dyno>>,
    time: Res<Time>,
) {
    for (mut dyno, mut tran) in dynos.iter_mut() {
        let mut point = tran.translation.truncate();
        let result = move_dyno_helper(dyno.as_mut(), &mut point, &rocks, time.delta_seconds());
        dyno.touching_rock = result.touched_rock;
        let z = tran.translation.z;
        tran.translation = point.extend(z);
    }
}

pub fn gravity_helper(
    dyno: &mut Dyno,
    point: &Vec2,
    fields: &Query<(&Field, &GlobalTransform), Without<Dyno>>,
    goal: &Query<(&Goal, &Transform)>,
    time_delta: f32,
) {
    let mut handle_field = |field: &Field, field_tran: &GlobalTransform| {
        let mult = field.effective_mult(point, &field_tran.translation().truncate(), dyno.radius);
        if mult > 0.00001 {
            dyno.vel *= 1.0 - field.drag;
            dyno.vel += field.dir * 400.0 * field.strength * mult * time_delta;
        }
    };
    for (field, field_tran) in fields.iter() {
        handle_field(field, field_tran);
    }
    let (goal, tran) = goal.single();
    if tran.translation.truncate().distance_squared(*point)
        < (dyno.radius + goal.radius) * (dyno.radius + goal.radius)
    {
        let diff = tran.translation.truncate() - *point;
        if diff.length_squared() > 0.001 {
            dyno.vel += diff.normalize() * goal.strength * 100.0;
        }
        // We're in the goal field!
        dyno.vel *= 1.0 - 0.03; // Increase drag
    }
}

pub fn apply_gravity(
    mut dynos: Query<(&mut Dyno, &Transform), Without<Field>>,
    fields: Query<(&Field, &GlobalTransform), Without<Dyno>>,
    goal: Query<(&Goal, &Transform)>,
    time: Res<Time>,
) {
    for (mut dyno, dyno_tran) in dynos.iter_mut() {
        gravity_helper(
            dyno.as_mut(),
            &dyno_tran.translation.truncate(),
            &fields,
            &goal,
            time.delta_seconds(),
        );
    }
}

pub fn should_apply_physics(gs: Res<GameState>) -> bool {
    match gs.meta {
        MetaState::Menu(_) => false,
        MetaState::Editor(editor_state) => match editor_state {
            EditorState::Testing => true,
            _ => false,
        },
        MetaState::Level(_) => true,
    }
}

pub fn update_avg_delta_time(mut adt: ResMut<AvgDeltaTime>, time: Res<Time>) {
    adt.update(time.delta_seconds());
}

pub fn register_physics(app: &mut App) {
    register_colliders(app);
    register_int_dynos(app);

    app.add_systems(Update, apply_gravity.run_if(should_apply_physics));
    app.add_systems(
        FixedUpdate,
        move_dynos.after(apply_gravity).run_if(should_apply_physics),
    );
    app.add_systems(Update, update_avg_delta_time);
    app.insert_resource(AvgDeltaTime::new());
}

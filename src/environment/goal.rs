use crate::{
    drawing::hollow::{CircleMarker, ShrinkingCircleBundle},
    meta::game_state::in_level,
    physics::dyno::IntDyno,
    ship::Ship,
};
use bevy::prelude::*;

#[derive(Event, Debug)]
pub struct GoalGet;

#[derive(Component)]
pub struct Goal {
    pub radius: f32,
    pub strength: f32,
    pub occupied_for: u32,
}

#[derive(Bundle)]
pub struct GoalBundle {
    pub goal: Goal,
    pub shrink: ShrinkingCircleBundle,
    pub spatial: SpatialBundle,
}
impl GoalBundle {
    fn new(pos: Vec2, radius: f32, strength: f32) -> Self {
        Self {
            goal: Goal {
                radius,
                strength,
                occupied_for: 0,
            },
            shrink: ShrinkingCircleBundle::new(radius, strength * 2.0),
            spatial: SpatialBundle::from_transform(Transform::from_translation(pos.extend(0.0))),
        }
    }

    pub fn spawn(pos: Vec2, commands: &mut Commands) {
        let goal = Self::new(pos, 30.0, 0.1);
        commands.spawn(goal).with_children(|comms| {
            comms.spawn((
                CircleMarker::new(30.0, Color::TOMATO),
                SpatialBundle::default(),
            ));
        });
    }
}

fn watch_for_goal_get(
    mut goals: Query<(&mut Goal, &GlobalTransform)>,
    ship: Query<(&GlobalTransform, &IntDyno), With<Ship>>,
    mut send_get: EventWriter<GoalGet>,
) {
    let Ok((stran, dyno)) = ship.get_single() else {
        return;
    };
    let Ok((mut goal, gtran)) = goals.get_single_mut() else {
        return;
    };
    let sp = stran.translation().truncate();
    let gp = gtran.translation().truncate();
    if sp.distance(gp) < dyno.radius && dyno.vel.length() < 25.0 {
        goal.occupied_for += 1;
        if goal.occupied_for == 40 {
            send_get.send(GoalGet {});
        }
    } else {
        goal.occupied_for = 0;
    }
}

pub fn register_goals(app: &mut App) {
    app.add_event::<GoalGet>();
    app.add_systems(Update, watch_for_goal_get.run_if(in_level));
}

use bevy::{prelude::*, render::view::RenderLayers};
use serde::{Deserialize, Serialize};

use crate::{
    drawing::{animated::AnimationStubs, layering::sprite_layer},
    environment::{
        goal::{GoalSize, GoalStrength},
        start::StartSize,
    },
    input::MouseState,
    physics::dyno::IntMoveable,
};

/// This is hard to make shared with point. At least share it between start/end
/// (or I'm just challenged)
#[derive(Component, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub(super) struct EStartGoalDragOffset(pub Option<IVec2>);

/// Can't tell if this is smart of challenged
#[derive(Component, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub(super) struct EStartGoalDiameter(pub u32);

#[derive(Component, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct EGoal {
    pub size: GoalSize,
    pub strength: GoalStrength,
}

#[derive(Bundle)]
pub(super) struct EGoalBundle {
    pub egoal: EGoal,
    pub animation: AnimationStubs,
    pub spatial: SpatialBundle,
    pub mv: IntMoveable,
    pub render_layers: RenderLayers,
    pub offset: EStartGoalDragOffset,
    pub diameter: EStartGoalDiameter, // NOTE: This is just to make the drag function less wordy. Will need a system to update if size changes later, doesn't exist yet, so not worrying
}

#[derive(Component, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct EStart {
    pub size: StartSize,
}

#[derive(Bundle)]
pub(super) struct EStartBundle {
    pub estart: EStart,
    pub animation: AnimationStubs,
    pub spatial: SpatialBundle,
    pub mv: IntMoveable,
    pub render_layers: RenderLayers,
    pub offset: EStartGoalDragOffset,
    pub diameter: EStartGoalDiameter,
}

pub(super) fn spawn_or_update_start_goal(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut starts: Query<&mut IntMoveable, (With<EStart>, Without<EGoal>)>,
    mut goals: Query<&mut IntMoveable, (With<EGoal>, Without<EStart>)>,
    mouse_state: Res<MouseState>,
) {
    let egoal = goals.get_single_mut();
    if keyboard.just_pressed(KeyCode::BracketRight) {
        match egoal {
            Ok(mut egoal) => {
                egoal.pos = mouse_state.world_pos.extend(0);
            }
            Err(_) => {
                commands.spawn(EGoalBundle {
                    egoal: EGoal::default(),
                    diameter: EStartGoalDiameter(EGoal::default().size.to_diameter()),
                    animation: AnimationStubs(vec![GoalSize::Medium.to_animation_bundle_stub()]),
                    spatial: SpatialBundle::default(),
                    mv: IntMoveable::new(mouse_state.world_pos.extend(0)),
                    render_layers: sprite_layer(),
                    offset: EStartGoalDragOffset(None),
                });
            }
        }
    }

    let estart = starts.get_single_mut();
    if keyboard.just_pressed(KeyCode::BracketLeft) {
        match estart {
            Ok(mut estart) => {
                estart.pos = mouse_state.world_pos.extend(0);
            }
            Err(_) => {
                commands.spawn(EStartBundle {
                    estart: EStart::default(),
                    diameter: EStartGoalDiameter(EStart::default().size.to_diameter()),
                    animation: AnimationStubs(vec![StartSize::Medium.to_animation_bundle_stub()]),
                    spatial: SpatialBundle::default(),
                    mv: IntMoveable::new(mouse_state.world_pos.extend(0)),
                    render_layers: sprite_layer(),
                    offset: EStartGoalDragOffset(None),
                });
            }
        }
    }
}

pub(super) fn start_goal_drag(
    mut starts: Query<
        (
            &mut IntMoveable,
            &mut EStartGoalDragOffset,
            &EStartGoalDiameter,
        ),
        (With<EStart>, Without<EGoal>),
    >,
    mut goals: Query<
        (
            &mut IntMoveable,
            &mut EStartGoalDragOffset,
            &EStartGoalDiameter,
        ),
        (With<EGoal>, Without<EStart>),
    >,
    mouse_state: Res<MouseState>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    let egoal = goals.get_single_mut();
    let estart = starts.get_single_mut();

    for thing in [egoal, estart] {
        if let Ok(mut thing) = thing {
            // First update the drag offsets
            if mouse_buttons.just_pressed(MouseButton::Left) {
                let center = thing.0.pos.truncate().as_vec2();
                let diameter = thing.2 .0 as f32;
                let dist_squared = center.distance_squared(mouse_state.world_pos.as_vec2());
                if dist_squared < (diameter / 2.0) * (diameter / 2.0) {
                    *thing.1 =
                        EStartGoalDragOffset(Some(thing.0.pos.truncate() - mouse_state.world_pos));
                }
            } else if !mouse_buttons.pressed(MouseButton::Left) {
                *thing.1 = EStartGoalDragOffset(None);
            }

            // Then move the points if there's a drag offset
            if let Some(offset) = thing.1 .0 {
                thing.0.pos.x = mouse_state.world_pos.x + offset.x;
                thing.0.pos.y = mouse_state.world_pos.y + offset.y;
            }
        }
    }
}

use bevy::{prelude::*, render::view::RenderLayers};
use serde::{Deserialize, Serialize};

use crate::{
    drawing::{animated::AnimationBundleStub, layering::sprite_layer},
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

#[derive(Clone, Debug, PartialEq, Default, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub(super) enum EGoalSize {
    #[default]
    Medium,
}
impl EGoalSize {
    pub fn to_diameter(&self) -> u32 {
        match *self {
            Self::Medium => 18,
        }
    }

    pub fn to_path(&self) -> String {
        match *self {
            Self::Medium => "sprites/start_goal/goal18.png".to_string(),
        }
    }

    pub fn length(&self) -> u8 {
        match *self {
            Self::Medium => 10,
        }
    }

    pub fn to_animation_bundle_stub(&self) -> AnimationBundleStub {
        let size = UVec2::new(self.to_diameter(), self.to_diameter());
        AnimationBundleStub::single_repeating(
            "shrinking",
            &self.to_path(),
            size,
            self.length(),
            None,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Default, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub(super) enum EStartSize {
    #[default]
    Medium,
}
impl EStartSize {
    pub fn to_diameter(&self) -> u32 {
        match *self {
            Self::Medium => 18,
        }
    }

    pub fn to_path(&self) -> String {
        match *self {
            Self::Medium => "sprites/start_goal/start18.png".to_string(),
        }
    }

    pub fn length(&self) -> u8 {
        match *self {
            Self::Medium => 10,
        }
    }

    pub fn to_animation_bundle_stub(&self) -> AnimationBundleStub {
        let size = UVec2::new(self.to_diameter(), self.to_diameter());
        AnimationBundleStub::single_repeating(
            "shrinking",
            &self.to_path(),
            size,
            self.length(),
            None,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Default, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub(super) enum EGoalStrength {
    #[default]
    Medium,
}
impl EGoalStrength {
    pub fn to_f32(&self) -> f32 {
        match *self {
            Self::Medium => 1.0,
        }
    }
}

#[derive(Component, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub(super) struct EGoal {
    pub size: EGoalSize,
    pub strength: EGoalStrength,
}

#[derive(Bundle)]
pub(super) struct EGoalBundle {
    pub egoal: EGoal,
    pub animation: AnimationBundleStub,
    pub mv: IntMoveable,
    pub render_layers: RenderLayers,
    pub offset: EStartGoalDragOffset,
    pub diameter: EStartGoalDiameter, // NOTE: This is just to make the drag function less wordy. Will need a system to update if size changes later, doesn't exist yet, so not worrying
}

#[derive(Component, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub(super) struct EStart {
    pub size: EStartSize,
}

#[derive(Bundle)]
pub(super) struct EStartBundle {
    pub estart: EStart,
    pub animation: AnimationBundleStub,
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
                    animation: EGoalSize::Medium.to_animation_bundle_stub(),
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
                    animation: EStartSize::Medium.to_animation_bundle_stub(),
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

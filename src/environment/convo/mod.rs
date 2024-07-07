use std::collections::VecDeque;

use bevy::prelude::*;
use data::ConvoKind;

use crate::{
    camera::{CameraMarker, CameraScale},
    physics::dyno::IntMoveable,
};

pub mod data;
mod operation;

/// Where the conversation box will appear.
#[derive(Component, Clone, Debug, Default)]
pub enum ConvoBoxPos {
    #[default]
    Default,
    // Free {
    //     top_left: UVec2,
    //     bot_right: UVec2,
    // },
}

/// Who is saying this
#[derive(Component, Clone, Debug, Default)]
pub enum ConvoBoxSpeaker {
    #[default]
    None, // I don't like doing this, but Option<T: Component> does not impl Component soo...
    Ship,
}

/// What are they saying, and the absolute camera movement (maybe none)
#[derive(Component, Clone, Debug)]
pub struct ConvoBoxContent {
    pub content: String,
    /// Start and end position for the camera while in this text box
    /// NOTE: If None, it will stay _whereever it already is_
    pub camera_mvmt: Option<(IVec2, IVec2)>,
    /// The scale the camera should have while in this text box
    /// NOTE: If None, it will stay _whatever it already is_
    pub camera_scale: Option<CameraScale>,
}

/// For controlling how long the text stays on screen, and marking it for replacement
#[derive(Component, Clone, Debug)]
pub struct ConvoBoxProgress {
    pub timer: Timer,
    /// "Finished" = timer done, "Absolutely finished" only true when timer done AND click/input/something
    /// So the player can click to finish the timer, then click to move on
    pub absolutely_finished: bool,
}

/// Everything that describes a convo box.
/// NOTE: This intentionally DOES NOT contain Transform, Name...
/// These babies will only be spawned by `Convo`s, inside oneshots.
/// That logic is responsible for placing and naming it properly
#[derive(Bundle, Clone, Debug)]
pub struct ConvoBoxBundle {
    pos: ConvoBoxPos,
    speaker: ConvoBoxSpeaker,
    content: ConvoBoxContent,
    progress: ConvoBoxProgress,
}
impl ConvoBoxBundle {
    const SECONDS_PER_CHAR: f32 = 0.1;

    pub fn new(speaker: ConvoBoxSpeaker, content: ConvoBoxContent) -> Self {
        Self {
            speaker,
            progress: {
                ConvoBoxProgress {
                    timer: Timer::from_seconds(
                        Self::SECONDS_PER_CHAR * content.content.len() as f32,
                        TimerMode::Once,
                    ),
                    absolutely_finished: false,
                }
            },
            content,
            pos: default(),
        }
    }

    pub fn force_duration(mut self, duration: f32) -> Self {
        self.progress.timer = Timer::from_seconds(duration, TimerMode::Once);
        self
    }
}

#[derive(Component, Clone, Debug)]
pub struct Convo {
    kind: ConvoKind,
    active_eid: Option<Entity>,
    bundles: VecDeque<ConvoBoxBundle>,
}

#[derive(Component, Reflect)]
struct CameraBeforeConvo(CameraMarker, IntMoveable);

#[derive(Event, Clone, Debug)]
pub struct StartConvo(pub ConvoKind);

#[derive(Event, Clone, Debug)]
pub struct ConvoEnded(pub ConvoKind);

pub struct ConvoPlugin;
impl Plugin for ConvoPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartConvo>();
        app.add_event::<ConvoEnded>();
        app.register_type::<CameraBeforeConvo>();

        data::register_convo_data(app);
        operation::register_convo_ops(app);
    }
}

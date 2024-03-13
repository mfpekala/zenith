use crate::camera::CameraMarker;
use crate::camera::CameraMode;
use crate::cutscenes::ChapterOneCutscenes;
use crate::cutscenes::Cutscene;
use crate::is_in_cutscene;
use crate::when_cutscene_started;
use bevy::prelude::*;

when_cutscene_started!(
    Cutscene::One(ChapterOneCutscenes::Alarm),
    when_entered_alarm
);
is_in_cutscene!(Cutscene::One(ChapterOneCutscenes::Alarm), is_in_alarm);

const ALARM_CAM_HOME: IVec2 = IVec2 {
    x: -10_000,
    y: -10_000,
};

pub(super) fn setup_alarm_cutscene(mut commands: Commands, mut cam_q: Query<&mut CameraMarker>) {
    let mut cam = cam_q.single_mut();
    cam.mode = CameraMode::Controlled;
    cam.pos = ALARM_CAM_HOME;
}

pub(super) fn update_alarm_cutscene() {}

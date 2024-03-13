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

pub(super) fn setup_alarm_cutscene() {
    println!("Setting up alarm cutscene");
}

pub(super) fn update_alarm_cutscene() {
    println!("Updating alarm cutscene");
}

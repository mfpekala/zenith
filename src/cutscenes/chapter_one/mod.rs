use bevy::prelude::*;

use self::alarm::{is_in_alarm, when_entered_alarm};

use super::translate_cutscenes;

pub mod alarm;
pub mod walk_to_work;

pub(super) fn register_chapter_one(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        alarm::setup_alarm_cutscene
            .after(translate_cutscenes)
            .run_if(when_entered_alarm),
    );
    app.add_systems(
        FixedUpdate,
        alarm::update_alarm_cutscene
            .after(translate_cutscenes)
            .after(alarm::setup_alarm_cutscene)
            .run_if(is_in_alarm),
    );
}

use self::{
    alarm::{is_in_alarm, update_alarm_cutscene, when_entered_alarm},
    walk_to_work::{is_in_walk_to_work, update_walk_to_work_cutscene, when_entered_walk_to_work},
};
use super::{is_in_any_cutscene, translate_cutscenes};
use crate::drawing::sunrise_mat::SunriseMaterialPlugin;
use bevy::prelude::*;

pub mod alarm;
pub mod walk_to_work;

pub(super) fn register_chapter_one(app: &mut App) {
    app.add_plugins(SunriseMaterialPlugin);

    /******* ALARM *******/
    app.add_systems(
        FixedUpdate,
        alarm::setup_alarm_cutscene
            .after(translate_cutscenes)
            .run_if(when_entered_alarm),
    );
    app.add_systems(
        FixedUpdate,
        alarm::update_alarm_cutscene
            .after(alarm::setup_alarm_cutscene)
            .run_if(is_in_alarm),
    );
    app.add_systems(
        FixedUpdate,
        alarm::stop_alarm_cutscene
            .after(update_alarm_cutscene)
            .run_if(is_in_any_cutscene),
    );

    /******* WALK TO WORK *******/
    app.add_systems(
        FixedUpdate,
        walk_to_work::setup_walk_to_work_cutscene
            .after(translate_cutscenes)
            .run_if(when_entered_walk_to_work),
    );
    app.add_systems(
        FixedUpdate,
        walk_to_work::update_walk_to_work_cutscene
            .after(walk_to_work::setup_walk_to_work_cutscene)
            .run_if(is_in_walk_to_work),
    );
    app.add_systems(
        FixedUpdate,
        walk_to_work::stop_walk_to_work_cutscene
            .after(update_walk_to_work_cutscene)
            .run_if(is_in_any_cutscene),
    );
}

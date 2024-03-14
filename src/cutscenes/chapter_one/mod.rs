use self::alarm::{is_in_alarm, when_entered_alarm};
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
            .after(translate_cutscenes)
            .after(alarm::setup_alarm_cutscene)
            .run_if(is_in_alarm),
    );
    app.add_systems(
        Update,
        alarm::stop_alarm_cutscene
            .after(translate_cutscenes)
            .run_if(is_in_any_cutscene),
    );

    /******* WALK TO WORK *******/
    // app.add_systems(
    //     FixedUpdate,
    //     walk_to_work::setup_walk_to_work_cutscene
    //         .after(translate_cutscenes)
    //         .run_if(when_entered_walk_to_work),
    // );
    // app.add_systems(
    //     FixedUpdate,
    //     walk_to_work::update_walk_to_work_cutscene
    //         .after(translate_cutscenes)
    //         .after(walk_to_work::setup_walk_to_work_cutscene)
    //         .run_if(is_in_walk_to_work),
    // );
    // app.add_systems(
    //     Update,
    //     walk_to_work::stop_walk_to_work_cutscene
    //         .after(translate_cutscenes)
    //         .run_if(is_in_any_cutscene),
    // );
}

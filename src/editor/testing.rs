use bevy::prelude::*;

use crate::{camera::CameraMarker, physics::dyno::IntMoveable};

const EDITING_HOME: IVec2 = IVec2::new(0, 0);
const TESTING_HOME: IVec2 = IVec2::new(6000, 6000);

pub(super) fn start_testing(
    world: &mut World,
    // mut camera_q: Query<&mut IntMoveable, With<CameraMarker>>
) {
    // let Ok(mut camera) = camera_q.get_single_mut() else {
    // return;
    // };
    println!("Yay! Started testing {:?}", world);
    // camera.pos = TESTING_HOME.extend(0);
}

pub(super) fn stop_testing(mut camera_q: Query<&mut IntMoveable, With<CameraMarker>>) {
    println!("Yay! Stopped testing");
    let Ok(mut camera) = camera_q.get_single_mut() else {
        return;
    };
    println!("Yay! Started testing");
    camera.pos = EDITING_HOME.extend(0);
}

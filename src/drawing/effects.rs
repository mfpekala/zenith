use crate::{
    camera::CameraMarker,
    math::{lerp, Spleen},
    meta::consts::WINDOW_WIDTH,
};
use bevy::prelude::*;

use super::layering::menu_layer;

#[derive(Component)]
pub struct EffectVal<const EFFECT_ID: i32> {
    start_val: f32,
    goal_val: f32,
    spleen: Spleen,
    timer: Timer,
}
impl<const EFFECT_ID: i32> EffectVal<EFFECT_ID> {
    pub fn blank(initial_val: f32) -> Self {
        Self {
            start_val: initial_val,
            goal_val: initial_val,
            spleen: Spleen::EaseInCubic,
            timer: Timer::from_seconds(0.0, TimerMode::Once),
        }
    }

    pub fn new(start_val: f32, end_val: f32, spleen: Spleen, duration: f32) -> Self {
        Self {
            start_val,
            goal_val: end_val,
            spleen,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn interp(&self) -> f32 {
        let x = self.spleen.interp(self.timer.fraction());
        lerp(x, self.start_val, self.goal_val)
    }
}

#[derive(Component)]
pub struct Sizeable {
    cur_val: f32,
}
impl Sizeable {
    pub const fn id() -> i32 {
        0
    }

    pub fn new() -> Self {
        Self { cur_val: 1.0 }
    }

    pub fn start_effect(
        &self,
        id: Entity,
        commands: &mut Commands,
        goal_val: f32,
        spleen: Spleen,
        duration: f32,
    ) {
        commands.entity(id).remove::<EffectVal<{ Self::id() }>>();
        commands.entity(id).insert(EffectVal::<{ Self::id() }>::new(
            self.cur_val,
            goal_val,
            spleen,
            duration,
        ));
    }
}

fn update_sizeables(
    mut commands: Commands,
    mut sizeables_q: Query<(
        Entity,
        &mut Transform,
        &mut Sizeable,
        Option<&mut EffectVal<{ Sizeable::id() }>>,
    )>,
    time: Res<Time>,
) {
    for (id, mut tran, mut sizeable, effect_opt) in sizeables_q.iter_mut() {
        match effect_opt {
            None => {
                // Spawn the effect value if it doesn't exist
                commands
                    .entity(id)
                    .insert(EffectVal::<{ Sizeable::id() }>::blank(sizeable.cur_val));
            }
            Some(mut effect_val) => {
                effect_val.timer.tick(time.delta());
                let val = effect_val.interp();
                sizeable.cur_val = val;
                tran.scale = (Vec2::ONE.extend(0.0)) * val;
            }
        }
    }
}

#[derive(Component)]
struct ZoomToBlack {
    cur_val: f32,
}
impl ZoomToBlack {
    const fn id() -> i32 {
        1
    }

    pub fn new() -> Self {
        Self { cur_val: 0.0 }
    }

    pub fn start_default_effect(
        &self,
        goal_val: f32,
        id: Entity,
        commands: &mut Commands,
        duration: f32,
    ) {
        commands.entity(id).remove::<EffectVal<{ Self::id() }>>();
        commands.entity(id).insert(EffectVal::<{ Self::id() }>::new(
            self.cur_val,
            goal_val,
            Spleen::EaseInOutCubic,
            duration,
        ));
    }
}

#[derive(Bundle)]
struct ZoomToBlackBundle {
    ztb: ZoomToBlack,
    effect_val: EffectVal<{ ZoomToBlack::id() }>,
    sprite: SpriteBundle,
}

fn setup_zoom_to_black(mut commands: Commands) {
    let mut clear_black = Color::BLACK;
    clear_black.set_a(0.0);
    commands
        .spawn(ZoomToBlackBundle {
            ztb: ZoomToBlack::new(),
            effect_val: EffectVal::blank(0.0),
            sprite: SpriteBundle {
                transform: Transform {
                    scale: Vec3::ONE * WINDOW_WIDTH as f32,
                    translation: Vec2::ZERO.extend(50.0),
                    ..default()
                },
                sprite: Sprite {
                    color: clear_black,
                    ..default()
                },
                ..default()
            },
        })
        .insert(menu_layer());
}

fn update_zoom_to_black(
    mut commands: Commands,
    mut ztb_q: Query<(
        Entity,
        &mut ZoomToBlack,
        &mut EffectVal<{ ZoomToBlack::id() }>,
        &mut Sprite,
    )>,
    mut cam: Query<&mut OrthographicProjection, With<CameraMarker>>,
    time: Res<Time>,
    mut triggers: EventReader<TriggerZoomToBlack>,
) {
    let Ok((id, mut ztb, mut effect_val, mut sprite)) = ztb_q.get_single_mut() else {
        return;
    };
    effect_val.timer.tick(time.delta());
    let val = effect_val.interp();
    ztb.cur_val = val;
    sprite.color.set_a(val);
    for mut cam in cam.iter_mut() {
        cam.scale = 1.0 / (1.0 + val * 10.0);
    }
    if let Some(trigger) = triggers.read().last() {
        ztb.start_default_effect(trigger.0 .0, id, &mut commands, trigger.0 .1);
    }
}

#[derive(Event)]
/// Just a thin wrapper around (goal, duration)
pub struct TriggerZoomToBlack(pub (f32, f32));

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TriggerZoomToBlack>();
        app.add_systems(Startup, setup_zoom_to_black);
        app.add_systems(Update, (update_sizeables, update_zoom_to_black));
    }
}

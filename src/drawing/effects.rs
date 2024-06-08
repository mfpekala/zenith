use std::collections::VecDeque;

use crate::{
    math::{lerp, Spleen},
    meta::consts::{MENU_HEIGHT, MENU_WIDTH},
};
use bevy::prelude::*;

use super::layering::menu_layer;

#[derive(Component)]
pub struct EffectVal {
    start_val: f32,
    goal_val: f32,
    spleen: Spleen,
    timer: Timer,
}
impl EffectVal {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenEffect {
    None,
    FadeToBlack,
    UnfadeToBlack,
}

#[derive(Resource)]
pub struct ScreenEffectManager {
    current_kind: ScreenEffect,
    current_val: EffectVal,
    queued_effects: VecDeque<ScreenEffect>,
}
impl ScreenEffectManager {
    fn blank() -> Self {
        Self {
            current_kind: ScreenEffect::None,
            current_val: EffectVal::blank(0.0),
            queued_effects: VecDeque::new(),
        }
    }

    pub fn queue_effect(&mut self, effect: ScreenEffect) {
        self.queued_effects.push_back(effect);
    }

    fn update_current(&mut self, time: &Res<Time>) {
        self.current_val.timer.tick(time.delta());
        if self.current_val.timer.finished()
            && !self.current_val.timer.just_finished()
            && (self.current_kind != ScreenEffect::None || self.queued_effects.len() > 0)
        {
            let kind = self
                .queued_effects
                .pop_front()
                .unwrap_or(ScreenEffect::None);
            let timer_time = if kind == ScreenEffect::None { 0.0 } else { 1.0 };
            self.current_kind = kind;
            self.current_val = EffectVal::new(0.0, 1.0, Spleen::EaseInOutCubic, timer_time);
        }
    }
}

#[derive(Component)]
struct ScreenBlackBox;

fn spawn_screen_black_box(mut commands: Commands) {
    commands.spawn((
        ScreenBlackBox,
        Name::new("screen_black_box"),
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(0.0, 0.0, 0.0, 0.0),
                ..default()
            },
            transform: Transform {
                scale: Vec3::new(MENU_WIDTH as f32, MENU_HEIGHT as f32, 1.0),
                translation: Vec3::new(0.0, 0.0, 100.0),
                ..default()
            },
            ..default()
        },
        menu_layer(),
    ));
}

fn manage_screen_effects(
    mut screen_effect: ResMut<ScreenEffectManager>,
    time: Res<Time>,
    mut black_box: Query<&mut Sprite, With<ScreenBlackBox>>,
) {
    screen_effect.update_current(&time);
    let mut black_box = black_box.single_mut();
    match screen_effect.current_kind {
        ScreenEffect::None => (),
        ScreenEffect::FadeToBlack => {
            let interp = screen_effect.current_val.interp();
            black_box.color = Color::rgba(0.0, 0.0, 0.0, interp);
        }
        ScreenEffect::UnfadeToBlack => {
            let interp = screen_effect.current_val.interp();
            black_box.color = Color::rgba(0.0, 0.0, 0.0, 1.0 - interp);
        }
    }
}

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScreenEffectManager::blank());
        app.add_systems(Startup, spawn_screen_black_box);
        app.add_systems(Update, manage_screen_effects);
    }
}

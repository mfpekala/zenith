use std::{collections::VecDeque, time::Duration};

use crate::{
    math::{lerp, Spleen},
    meta::{
        consts::{MENU_HEIGHT, MENU_WIDTH},
        game_state::{GameState, SetMetaState, SetPaused},
    },
};
use bevy::prelude::*;

use super::layering::menu_layer;

#[derive(Component)]
pub struct EffectVal {
    start_val: f32,
    goal_val: f32,
    spleen: Spleen,
    pub timer: Timer,
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

    pub fn get_start_val(&self) -> f32 {
        self.start_val
    }

    pub fn get_goal_val(&self) -> f32 {
        self.goal_val
    }

    pub fn get_spleen(&self) -> Spleen {
        self.spleen
    }

    pub fn interp(&self) -> f32 {
        let x = self.spleen.interp(self.timer.fraction());
        lerp(x, self.start_val, self.goal_val)
    }

    pub fn interp_time(&self) -> f32 {
        self.spleen.interp(self.timer.fraction())
    }

    pub fn just_finished(&self) -> bool {
        self.timer.just_finished()
    }

    pub fn finished(&self) -> bool {
        self.timer.finished()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScreenEffect {
    None,
    FadeToBlack(Option<GameState>),
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
            let timer_time = if kind == ScreenEffect::None { 0.0 } else { 0.3 };
            self.current_kind = kind;
            self.current_val = EffectVal::new(0.0, 1.0, Spleen::EaseInOutCubic, timer_time);
        }
    }

    pub fn is_none(&self) -> bool {
        self.current_kind == ScreenEffect::None
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
    mut meta_writer: EventWriter<SetMetaState>,
    mut pause_writer: EventWriter<SetPaused>,
) {
    screen_effect.update_current(&time);
    let mut black_box = black_box.single_mut();
    match &screen_effect.current_kind {
        ScreenEffect::None => (),
        ScreenEffect::FadeToBlack(gs) => {
            let interp = screen_effect.current_val.interp();
            black_box.color = Color::rgba(0.0, 0.0, 0.0, interp);
            if interp >= 0.9999 {
                if let Some(gs) = gs {
                    meta_writer.send(SetMetaState(gs.meta.clone()));
                    pause_writer.send(SetPaused(gs.pause));
                }
            }
        }
        ScreenEffect::UnfadeToBlack => {
            if black_box.color.a() < 0.0001 {
                black_box.color.set_a(0.0);
                screen_effect.current_val.timer.tick(Duration::new(100, 0));
            } else {
                let interp = screen_effect.current_val.interp();
                black_box.color = Color::rgba(0.0, 0.0, 0.0, 1.0 - interp);
            }
        }
    }
}

fn tick_effects(mut effects: Query<&mut EffectVal>, time: Res<Time>) {
    for mut effect in effects.iter_mut() {
        effect.timer.tick(time.delta());
    }
}

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScreenEffectManager::blank());
        app.add_systems(Startup, spawn_screen_black_box);
        app.add_systems(Update, manage_screen_effects);
        app.add_systems(PreUpdate, tick_effects);
    }
}

use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    camera::CameraMarker,
    drawing::{
        effects::{EffectVal, Sizeable},
        BgLightGizmoGroup, BgSpriteGizmoGroup,
    },
    math::Spleen,
    menu::constellation_screen::ConstellationScreenData,
    meta::consts::{SCREEN_HEIGHT, SCREEN_WIDTH, WINDOW_HEIGHT, WINDOW_WIDTH},
};

pub const MAX_HYPERSPACE_SPEED: i32 = 20_000;

#[derive(Component)]
pub struct Star {
    pub color: Color,
    pub depth: u8,
    pub pos: Vec2,
    pub size: f32,
    pub twinkling: bool,
    pub brightness: f32,
}
impl Star {
    pub const fn min_depth() -> u8 {
        12
    }

    pub const fn max_depth() -> u8 {
        24
    }

    pub fn dmult(&self) -> f32 {
        (self.depth as f32) * (1.3 as f32).powi(self.depth as i32)
    }

    pub fn max_dmult() -> f32 {
        (Self::max_depth() as f32) * (1.3 as f32).powi(Self::max_depth() as i32)
    }
}

#[derive(Bundle)]
pub struct StarBundle {
    pub star: Star,
}

#[derive(Resource)]
pub struct StarOrder(pub Vec<Entity>);

#[derive(Resource, Debug)]
pub struct HyperSpace {
    pub offset: IVec2,
    pub old_speed: IVec2,
    pub cur_speed: IVec2,
    pub goal_speed: IVec2,
    pub spleen: Spleen,
    pub timer: Timer,
}
impl HyperSpace {
    pub fn approach_speed(&mut self, goal: IVec2, time: f32, spleen: Spleen) {
        self.old_speed = self.cur_speed;
        self.goal_speed = goal;
        self.timer = Timer::from_seconds(time, TimerMode::Once);
        self.spleen = spleen;
    }

    pub fn interp(&self) -> f32 {
        self.spleen.interp(self.timer.fraction())
    }
}

fn spawn_test_stars(mut commands: Commands, mut star_order: ResMut<StarOrder>) {
    let mut rng = thread_rng();
    let mut dni = vec![];
    for _ in 0..200 {
        let depth = rng.gen_range(Star::min_depth()..=Star::max_depth());
        let star = Star {
            color: Color::Hsla {
                hue: 20.0 + rng.gen::<f32>() * 180.0,
                saturation: 0.3 + rng.gen::<f32>() * 0.3,
                lightness: 0.3 + rng.gen::<f32>() * 0.6,
                alpha: 1.0,
            },
            depth,
            pos: Vec2 {
                x: rng.gen::<f32>() * SCREEN_WIDTH as f32,
                y: rng.gen::<f32>() * SCREEN_HEIGHT as f32,
            },
            size: 1.0 + rng.gen::<f32>(),
            twinkling: false,
            brightness: 0.3,
        };
        let id = commands.spawn(StarBundle { star }).id();
        dni.push((depth, id));
    }
    dni.sort_by_key(|e| 0 - e.0 as i16);
    *star_order = StarOrder(dni.into_iter().map(|pair| pair.1).collect());
}

fn depth_to_size_mult(depth: u8) -> f32 {
    0.2 + 3.6 * (0.75 as f32).powi(depth as i32 - 12)
}

fn draw_star(
    star: &Star,
    final_pos: Vec2,
    bg_sprite_gz: &mut Gizmos<BgSpriteGizmoGroup>,
    bg_light_gz: &mut Gizmos<BgLightGizmoGroup>,
    lightness_boost: f32,
) {
    let up = Vec2 {
        x: 0.0,
        y: star.size,
    } * depth_to_size_mult(star.depth);
    let right = Vec2 {
        x: star.size,
        y: 0.0,
    } * depth_to_size_mult(star.depth);
    let white = Color::Hsla {
        hue: star.color.h(),
        saturation: 1.0,
        lightness: star.brightness + lightness_boost,
        alpha: 1.0,
    };
    for diff in [up, right, up * 0.6 + right * 0.6, up * 0.6 - right * 0.6].iter() {
        let calc_diff = *diff * (1.0 + 1.5 * lightness_boost);
        bg_sprite_gz.line_2d(final_pos - calc_diff, final_pos + calc_diff, star.color);
        bg_light_gz.line_2d(final_pos - calc_diff, final_pos + calc_diff, white);
    }
}

fn update_hyper_space(mut hyperspace: ResMut<HyperSpace>, time: Res<Time>) {
    let cur_speed = hyperspace.cur_speed;
    hyperspace.offset += cur_speed;
    let max_dmult_i32: i32 = Star::max_dmult().ceil() as i32;
    hyperspace.offset.x = hyperspace
        .offset
        .x
        .rem_euclid(WINDOW_WIDTH as i32 * max_dmult_i32);
    hyperspace.offset.y = hyperspace
        .offset
        .y
        .rem_euclid(WINDOW_HEIGHT as i32 * max_dmult_i32);
    let x = hyperspace.interp();
    let tmp_speed = hyperspace.old_speed.as_vec2()
        + x * (hyperspace.goal_speed.as_vec2() - hyperspace.old_speed.as_vec2());
    hyperspace.cur_speed = IVec2::new(tmp_speed.x.round() as i32, tmp_speed.y.round() as i32);
    hyperspace.timer.tick(time.delta());
}

fn update_stars(
    mut stars: Query<&mut Star>,
    mut bg_sprite_gz: Gizmos<BgSpriteGizmoGroup>,
    mut bg_light_gz: Gizmos<BgLightGizmoGroup>,
    cam_q: Query<&CameraMarker>,
    star_order: Res<StarOrder>,
    hyperspace: Res<HyperSpace>,
    constel_data: Query<&ConstellationScreenData>,
    texts: Query<(&Text, &EffectVal<{ Sizeable::id() }>)>,
) {
    let Ok(cam) = cam_q.get_single() else {
        return;
    };

    // Draw in the stars
    let mut rng = thread_rng();
    let lightness_boost = ((hyperspace.cur_speed / MAX_HYPERSPACE_SPEED).length_squared() as f32)
        .sqrt()
        .min(0.6);
    let mut left_val = 0.0;
    let mut right_val = 0.0;
    for (text, eval) in texts.iter() {
        // Subtract one because the scaling animations go from 1 -> 1.3 (ish)
        if text.sections[0].value == "A" {
            left_val = eval.interp() - 1.0;
        }
        if text.sections[0].value == "B" {
            right_val = eval.interp() - 1.0;
        }
    }
    for star_id in star_order.0.iter() {
        let Ok(mut star) = stars.get_mut(*star_id) else {
            continue;
        };
        let screen_size = Vec2 {
            x: SCREEN_WIDTH as f32,
            y: SCREEN_HEIGHT as f32,
        };
        let ref_screen_size = screen_size * star.dmult();
        let frac = (-cam.pos.as_vec2() + hyperspace.offset.as_vec2() + star.pos * star.dmult())
            .rem_euclid(ref_screen_size)
            / ref_screen_size;
        let buff_frac = 0.33;
        let offset = screen_size * (1.0 + 2.0 * buff_frac) * frac - screen_size * (1.0 + buff_frac);
        // Maybe this looks better rounded? idk
        let final_pos = offset + screen_size / 2.0;

        // Hack in our cool save file picking strategy
        let mut additional_lightness = 0.0;
        if constel_data.iter().len() > 0 {
            additional_lightness += if frac.x < 0.5 { left_val } else { right_val };
        }

        draw_star(
            &star,
            final_pos,
            &mut bg_sprite_gz,
            &mut bg_light_gz,
            lightness_boost + additional_lightness,
        );
        if star.twinkling {
            star.brightness = (star.brightness + 0.001).min(0.7);
            if rng.gen_bool(0.01) {
                star.twinkling = false;
            }
        } else {
            star.brightness = (star.brightness - 0.001).max(0.3);
            if rng.gen_bool(0.0005) {
                star.twinkling = true;
            }
        }
    }
}

pub const BASE_TITLE_HYPERSPACE_SPEED: IVec2 = IVec2 { x: 20, y: 1 };

pub fn register_background(app: &mut App) {
    app.insert_resource(StarOrder(vec![]));
    app.insert_resource(HyperSpace {
        offset: IVec2::ZERO,
        old_speed: BASE_TITLE_HYPERSPACE_SPEED,
        cur_speed: BASE_TITLE_HYPERSPACE_SPEED,
        goal_speed: BASE_TITLE_HYPERSPACE_SPEED,
        spleen: Spleen::EaseInOutQuintic,
        timer: Timer::from_seconds(0.0, TimerMode::Once),
    });
    app.add_systems(Startup, spawn_test_stars);
    app.add_systems(Update, update_stars);
    app.add_systems(Update, update_hyper_space);
}

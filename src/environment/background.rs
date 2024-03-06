use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    camera::CameraMarker,
    drawing::{BgLightGizmoGroup, BgSpriteGizmoGroup},
    meta::consts::{WINDOW_HEIGHT, WINDOW_WIDTH},
};

#[derive(Component)]
pub struct Star {
    pub color: Color,
    pub depth: u8,
    pub pos: Vec2,
    pub size: f32,
}

#[derive(Bundle)]
pub struct StarBundle {
    pub star: Star,
}

fn spawn_test_stars(mut commands: Commands) {
    let mut rng = thread_rng();
    for _ in 0..300 {
        let star = Star {
            color: Color::Hsla {
                hue: 220.0,
                saturation: 1.0,
                lightness: 0.6,
                alpha: 1.0,
            },
            depth: rng.gen_range(12..=24),
            pos: Vec2 {
                x: rng.gen::<f32>() * WINDOW_WIDTH as f32,
                y: rng.gen::<f32>() * WINDOW_HEIGHT as f32,
            },
            size: 0.5 + rng.gen::<f32>(),
        };
        commands.spawn(StarBundle { star });
    }
}

fn depth_to_size_mult(depth: u8) -> f32 {
    2.5 + 30.0 * (0.95 as f32).powi(depth as i32)
}

fn draw_star(
    star: &Star,
    final_pos: Vec2,
    bg_sprite_gz: &mut Gizmos<BgSpriteGizmoGroup>,
    bg_light_gz: &mut Gizmos<BgLightGizmoGroup>,
) {
    let up = Vec2 {
        x: 0.0,
        y: star.size,
    } * depth_to_size_mult(star.depth);
    let right = Vec2 {
        x: star.size,
        y: 0.0,
    } * depth_to_size_mult(star.depth);
    let mut rng = thread_rng();
    let lite_color = Color::Hsla {
        hue: 220.0,
        saturation: 1.0,
        lightness: 0.6 * (rng.gen::<f32>() * 0.1 + 0.9),
        alpha: 1.0,
    };
    for diff in [up, right, up * 0.6 + right * 0.6, up * 0.6 - right * 0.6].iter() {
        bg_sprite_gz.line_2d(final_pos - *diff, final_pos + *diff, lite_color);
        bg_light_gz.line_2d(final_pos - *diff, final_pos + *diff, lite_color);
    }
}

fn update_stars(
    stars: Query<&Star>,
    mut bg_sprite_gz: Gizmos<BgSpriteGizmoGroup>,
    mut bg_light_gz: Gizmos<BgLightGizmoGroup>,
    cam_q: Query<&CameraMarker>,
) {
    let Ok(cam) = cam_q.get_single() else {
        return;
    };
    for star in stars.iter() {
        let window_size = Vec2 {
            x: WINDOW_WIDTH as f32,
            y: WINDOW_HEIGHT as f32,
        };
        let ref_window_size = window_size * star.depth as f32;
        let frac = (cam.fake_pos * (0.0 - 1.0) + star.pos * star.depth as f32)
            .rem_euclid(ref_window_size)
            / ref_window_size;
        let buff_frac = 0.33;
        let offset = window_size * (1.0 + 2.0 * buff_frac) * frac - window_size * (1.0 + buff_frac);
        let final_pos = offset + window_size / 2.0;
        draw_star(star, final_pos, &mut bg_sprite_gz, &mut bg_light_gz);
    }
}

pub fn register_background(app: &mut App) {
    app.add_systems(Startup, spawn_test_stars);
    app.add_systems(Update, update_stars);
}

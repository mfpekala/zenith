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
    pub twinkling: bool,
    pub brightness: f32,
}

#[derive(Bundle)]
pub struct StarBundle {
    pub star: Star,
}

#[derive(Resource)]
pub struct StarOrder(pub Vec<Entity>);

fn spawn_test_stars(mut commands: Commands, mut star_order: ResMut<StarOrder>) {
    let mut rng = thread_rng();
    let mut dni = vec![];
    for _ in 0..300 {
        let depth = rng.gen_range(12..=24);
        let star = Star {
            color: Color::Hsla {
                hue: 20.0 + rng.gen::<f32>() * 180.0,
                saturation: 0.3 + rng.gen::<f32>() * 0.3,
                lightness: 0.3 + rng.gen::<f32>() * 0.3,
                alpha: 1.0,
            },
            depth,
            pos: Vec2 {
                x: rng.gen::<f32>() * WINDOW_WIDTH as f32,
                y: rng.gen::<f32>() * WINDOW_HEIGHT as f32,
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
    5.0 + 15.0 * (0.75 as f32).powi(depth as i32 - 12)
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
    let white = Color::Hsla {
        hue: star.color.h(),
        saturation: 1.0,
        lightness: star.brightness,
        alpha: 1.0,
    };
    for diff in [up, right, up * 0.6 + right * 0.6, up * 0.6 - right * 0.6].iter() {
        bg_sprite_gz.line_2d(final_pos - *diff, final_pos + *diff, star.color);
        bg_light_gz.line_2d(final_pos - *diff, final_pos + *diff, white);
    }
}

fn update_stars(
    mut stars: Query<&mut Star>,
    mut bg_sprite_gz: Gizmos<BgSpriteGizmoGroup>,
    mut bg_light_gz: Gizmos<BgLightGizmoGroup>,
    cam_q: Query<&CameraMarker>,
    star_order: Res<StarOrder>,
) {
    let Ok(cam) = cam_q.get_single() else {
        return;
    };
    let mut rng = thread_rng();
    for star_id in star_order.0.iter() {
        let Ok(mut star) = stars.get_mut(*star_id) else {
            continue;
        };
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
        draw_star(&star, final_pos, &mut bg_sprite_gz, &mut bg_light_gz);
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

pub fn register_background(app: &mut App) {
    app.insert_resource(StarOrder(vec![]));
    app.add_systems(Startup, spawn_test_stars);
    app.add_systems(Update, update_stars);
}

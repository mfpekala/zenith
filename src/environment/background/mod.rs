use crate::{
    drawing::layering::bg_sprite_layer,
    math::Spleen,
    meta::consts::{TuneableConsts, SCREEN_HEIGHT, SCREEN_WIDTH},
};
use bevy::prelude::*;

#[derive(Component, Clone)]
/// A struct to mark things as being in the background layer for clearing purposes
pub struct BgMarker;

#[derive(Component, Clone, Debug, Default)]
pub struct BgDepth {
    /// The logical depth of the entity. Effects position of the element.
    pub depth: u8,
    /// Should the entity shrink according to it's depth?
    pub shrink: Option<bool>,
    /// Should the final rendered position of the element snap to pixel boundaries?
    pub snap_to_pixel: Option<bool>,
    /// If set, the position of this element will wrap every `x` copies of SCREEN.
    /// I.e., if it is Some(1.0), then it will always be on screen, Some(2) will be on screen half the time, etc.
    pub wrap: Option<f32>,
}
impl BgDepth {
    /// Exponential multiplier to be used in calculating depth-to-translation
    pub fn dmult_translation(&self, tune: &Res<TuneableConsts>) -> f32 {
        let base = tune
            .map
            .get("bg_dmult_translation_base")
            .unwrap_or(&1.3)
            .clone();
        (self.depth as f32) * (base as f32).powi(self.depth as i32)
    }

    /// Exponential multiplier to be used in calculating depth-to-scale
    pub fn dmult_scale(&self, tune: &Res<TuneableConsts>) -> f32 {
        let base = tune.map.get("bg_dmult_scale_base").unwrap_or(&1.3).clone();
        (base as f32).powi(self.depth as i32)
    }
}

#[derive(Component, Clone, Debug, Default)]
/// Component driving the positioning of background elements, along with BgDepth
pub struct BgOffset {
    pub vel: Vec2,
    pub pos: IVec2,
    pub placed: Vec2,
    pub rem: Vec2,
    pub base_scale: f32,
    pub tweak_scale: Option<f32>,
}
impl BgOffset {
    pub fn from_frac_pos(
        tune: &Res<TuneableConsts>,
        bg_depth: &BgDepth,
        frac_pos: Vec2,
        scale: f32,
    ) -> Self {
        let og_screen_size = Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);
        let ref_screen_size =
            og_screen_size * bg_depth.dmult_translation(tune) * bg_depth.wrap.unwrap_or(1.0);
        let fpos = ref_screen_size * frac_pos;
        let adjusted_pos = IVec2 {
            x: fpos.x as i32,
            y: fpos.y as i32,
        };
        BgOffset {
            pos: adjusted_pos,
            base_scale: scale,
            ..default()
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct BgOffsetSpleen {
    pub vel_start: Vec2,
    pub vel_goal: Vec2,
    pub spleen: Spleen,
    pub timer: Timer,
}

#[derive(Bundle, Clone)]
pub struct PlacedBgBundle {
    pub _marker: BgMarker,
    pub depth: BgDepth,
    pub offset: BgOffset,
}
impl PlacedBgBundle {
    /// Creates a bg bundle at the given fraction of the screen (should be between 0.5 and -0.5)
    pub fn basic_stationary(
        tune: &Res<TuneableConsts>,
        depth: u8,
        frac_pos: Vec2,
        scale: f32,
    ) -> Self {
        let depth = BgDepth {
            depth,
            shrink: Some(true),
            snap_to_pixel: Some(true),
            wrap: Some(1.5),
        };
        let offset = BgOffset::from_frac_pos(tune, &depth, frac_pos, scale);
        Self {
            _marker: BgMarker,
            depth,
            offset,
        }
    }
}

fn _test_new_bg_system_startup(mut commands: Commands) {
    let max_d: u8 = 24;
    for depth in 0..24 {
        let bg_depth = BgDepth {
            depth,
            wrap: Some(1.2),
            snap_to_pixel: Some(true),
            shrink: Some(true),
            ..default()
        };
        let frac = (depth as f32) / (max_d as f32);
        commands.spawn((
            BgOffset {
                vel: Vec2::new(40.0, 0.0),
                pos: IVec2::new(0, (-80 + (160 as f32 * frac) as i32) * 10),
                base_scale: 80.0,
                ..default()
            },
            bg_depth,
            SpriteBundle {
                transform: Transform { ..default() },
                ..default()
            },
            bg_sprite_layer(),
        ));
    }
}

fn move_bg_entities(
    mut commands: Commands,
    mut objects: Query<(
        Entity,
        &mut Transform,
        &BgDepth,
        &mut BgOffset,
        Option<&mut BgOffsetSpleen>,
    )>,
    tune: Res<TuneableConsts>,
    time: Res<Time>,
) {
    let og_screen_size = Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);
    for (id, mut tran, depth, mut offset, spleen) in objects.iter_mut() {
        // We move the objects in much the same way that we move dynos
        let would_move = offset.vel + offset.rem;
        let move_x = would_move.x.round() as i32;
        let move_y = would_move.y.round() as i32;
        if move_x != 0 {
            offset.pos.x += move_x;
            offset.rem.x = would_move.x - move_x as f32;
        } else {
            offset.rem.x = would_move.x;
        }
        if move_y != 0 {
            offset.pos.y += move_y;
            offset.rem.y = would_move.y - move_y as f32;
        } else {
            offset.rem.y = would_move.y;
        }
        // Then place them on screen accounting for their "depth"
        let dmult_translation = depth.dmult_translation(&tune);
        let ref_screen_size = og_screen_size * dmult_translation;
        // Wrap if needed
        if let Some(wrap) = depth.wrap {
            if offset.pos.as_vec2().x.abs() > ref_screen_size.x * wrap * 0.5 {
                let rem = (offset.pos.x as f32).rem_euclid(ref_screen_size.x);
                offset.pos.x = (-ref_screen_size.x * wrap + rem) as i32;
            }
            if offset.pos.as_vec2().y.abs() > ref_screen_size.y * wrap * 0.5 {
                let rem = (offset.pos.y as f32).rem_euclid(ref_screen_size.y);
                offset.pos.y = (-ref_screen_size.y * wrap + rem) as i32;
            }
        }
        // The relative position on the screen. If on the screen, x and y will be in (-0.5, 0.5)
        let ref_frac = offset.pos.as_vec2() / ref_screen_size;
        // Then we scale it up to find an actual position which it should live
        let mut pos = ref_frac * og_screen_size;
        if depth.snap_to_pixel.unwrap_or(false) {
            pos = pos.round();
        }
        offset.placed = pos;
        let z = -(depth.depth as f32);
        tran.translation = pos.extend(z);
        if depth.shrink.unwrap_or(false) {
            let factor = depth.dmult_scale(&tune);
            tran.scale =
                (Vec2::ONE * factor * offset.base_scale * offset.tweak_scale.unwrap_or(1.0))
                    .extend(z);
        }
        // Finally we apply spleen
        if let Some(mut spleen) = spleen {
            spleen.timer.tick(time.delta());
            if spleen.timer.finished() {
                commands.entity(id).remove::<BgOffsetSpleen>();
            }
            let frac = spleen.spleen.interp(spleen.timer.fraction());
            offset.vel = spleen.vel_start + (spleen.vel_goal - spleen.vel_start) * frac;
        }
    }
}

pub fn clear_background_entities(commands: &mut Commands, bgs: &Query<Entity, With<BgMarker>>) {
    for id in bgs.iter() {
        commands.entity(id).despawn_recursive();
    }
}

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, test_new_bg_system_startup);
        app.add_systems(FixedUpdate, move_bg_entities);
    }
}

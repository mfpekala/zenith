use crate::{
    camera::{camera_movement, CameraMarker},
    drawing::layering::{bg_light_layer, bg_sprite_layer},
    math::Spleen,
    meta::consts::{SCREEN_HEIGHT, SCREEN_WIDTH},
    physics::{dyno::IntMoveable, BulletTime},
};
use bevy::prelude::*;
use rand::{thread_rng, Rng};

#[derive(Resource, PartialEq, Copy, Clone)]
pub enum BackgroundKind {
    None,
    ParallaxStars(usize),
}

#[derive(Resource)]
struct LastBackgroundKind(BackgroundKind);

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
    pub fn dmult_translation(&self) -> f32 {
        let base = 1.4;
        (self.depth as f32) * (base as f32).powi(self.depth as i32)
    }

    /// Exponential multiplier to be used in calculating depth-to-scale
    pub fn dmult_scale(&self) -> f32 {
        let base = 0.8;
        (base as f32).powi(self.depth as i32)
    }
}

#[derive(Clone, Debug)]
pub enum BgPosKind {
    Free(IVec2),
    Parallax(IVec2),
}
impl Default for BgPosKind {
    fn default() -> Self {
        Self::Free(IVec2::ZERO)
    }
}
impl BgPosKind {
    fn pos(&self) -> IVec2 {
        match self {
            &Self::Free(pos) => pos,
            &Self::Parallax(pos) => pos,
        }
    }

    fn set_pos(&mut self, new_pos: IVec2) {
        *self = match self {
            Self::Free(_) => Self::Free(new_pos),
            Self::Parallax(_) => Self::Parallax(new_pos),
        };
    }

    fn add_ivec(&mut self, ivec: IVec2) {
        let new_pos = self.pos() + ivec;
        self.set_pos(new_pos);
    }
}

#[derive(Component, Clone, Debug, Default)]
/// Component driving the positioning of background elements, along with BgDepth
pub struct BgOffset {
    pub vel: Vec2,
    pub pos: BgPosKind,
    pub placed: Vec2,
    pub rem: Vec2,
    pub base_scale: f32,
    pub tweak_scale: Option<f32>,
}
impl BgOffset {
    pub fn from_frac_pos(
        bg_depth: &BgDepth,
        frac_pos: Vec2,
        scale: f32,
        is_parallax: bool,
    ) -> Self {
        let og_screen_size = Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);
        let ref_screen_size =
            og_screen_size * bg_depth.dmult_translation() * bg_depth.wrap.unwrap_or(1.0);
        let fpos = ref_screen_size * frac_pos;
        let adjusted_pos = IVec2 {
            x: fpos.x as i32,
            y: fpos.y as i32,
        };
        let pos = if is_parallax {
            BgPosKind::Parallax(adjusted_pos)
        } else {
            BgPosKind::Free(adjusted_pos)
        };
        BgOffset {
            pos,
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
    /// This bundle will have pos-type Free and will NOT have a parallax effect
    pub fn basic_stationary(depth: u8, frac_pos: Vec2, scale: f32) -> Self {
        let depth = BgDepth {
            depth,
            shrink: Some(true),
            snap_to_pixel: Some(true),
            wrap: Some(3.0),
        };
        let offset = BgOffset::from_frac_pos(&depth, frac_pos, scale, false);
        Self {
            _marker: BgMarker,
            depth,
            offset,
        }
    }

    /// Creates a bg bundle at the given fraction of the screen
    /// This bundle will have pos-type Parallax and will have a parallax effect
    pub fn basic_parallax(depth: u8, frac_pos: Vec2, scale: f32) -> Self {
        let depth = BgDepth {
            depth,
            shrink: Some(true),
            snap_to_pixel: Some(true),
            wrap: Some(3.0),
        };
        let offset = BgOffset::from_frac_pos(&depth, frac_pos, scale, true);
        Self {
            _marker: BgMarker,
            depth,
            offset,
        }
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
    time: Res<Time>,
    bullet_time: Res<BulletTime>,
    camera: Query<&IntMoveable, With<CameraMarker>>,
) {
    let Ok(camera) = camera.get_single() else {
        error!("Weird stuff in move bg entities with camera");
        return;
    };
    let og_screen_size = Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);
    let wrap_pos = |mut pos: IVec2, ref_screen_size: Vec2, wrap_num: f32| -> IVec2 {
        if pos.as_vec2().x.abs() > ref_screen_size.x * wrap_num * 0.5 {
            let rem = (pos.x as f32).rem_euclid(ref_screen_size.x);
            pos.x = (-ref_screen_size.x * wrap_num * 0.5 + rem) as i32;
        }
        if pos.as_vec2().y.abs() > ref_screen_size.y * wrap_num * 0.5 {
            let rem = (pos.y as f32).rem_euclid(ref_screen_size.y);
            pos.y = (-ref_screen_size.y * wrap_num * 0.5 + rem) as i32;
        }
        pos
    };
    let wrapped_camera = wrap_pos(camera.pos.truncate(), og_screen_size * 100.0, 3.0);
    for (id, mut tran, depth, mut offset, spleen) in objects.iter_mut() {
        // We move the objects in much the same way that we move dynos
        let would_move = offset.vel + offset.rem;
        let move_x = would_move.x.round() as i32;
        let move_y = would_move.y.round() as i32;
        offset.pos.add_ivec(IVec2::new(move_x, move_y));
        if move_x != 0 {
            offset.rem.x = would_move.x - move_x as f32;
        } else {
            offset.rem.x = would_move.x;
        }
        if move_y != 0 {
            offset.rem.y = would_move.y - move_y as f32;
        } else {
            offset.rem.y = would_move.y;
        }
        // Then place them on screen accounting for their "depth"
        let dmult_translation = depth.dmult_translation();
        let ref_screen_size = og_screen_size * dmult_translation;
        if let Some(wrap) = depth.wrap {
            let new_pos = wrap_pos(offset.pos.pos(), ref_screen_size, wrap);
            offset.pos.set_pos(new_pos);
        }
        let placing_pos = match offset.pos {
            BgPosKind::Free(pos) => pos,
            BgPosKind::Parallax(pos) => pos - wrapped_camera,
        };
        // The relative position on the screen. If on the screen, x and y will be in (-0.5, 0.5)
        let ref_frac = placing_pos.as_vec2() / ref_screen_size;
        // Then we scale it up to find an actual position which it should live
        let mut pos = ref_frac * og_screen_size;
        if depth.snap_to_pixel.unwrap_or(false) {
            pos = pos.round();
        }
        offset.placed = pos;
        let z = -(depth.depth as f32);
        tran.translation = pos.extend(z);
        if depth.shrink.unwrap_or(false) {
            let factor = depth.dmult_scale();
            tran.scale =
                (Vec2::ONE * factor * offset.base_scale * offset.tweak_scale.unwrap_or(1.0))
                    .extend(z);
        }
        // Finally we apply spleen
        if let Some(mut spleen) = spleen {
            spleen
                .timer
                .tick(time.delta().mul_f32(bullet_time.factor()));
            if spleen.timer.finished() {
                commands.entity(id).remove::<BgOffsetSpleen>();
            }
            let frac = spleen.spleen.interp(spleen.timer.fraction());
            offset.vel = spleen.vel_start + (spleen.vel_goal - spleen.vel_start) * frac;
        }
    }
}

#[derive(Component)]
pub struct BackgroundRoot;

fn setup_background(mut commands: Commands) {
    commands.spawn((
        Name::new("BackgroundRoot"),
        BackgroundRoot,
        SpatialBundle::default(),
    ));
}

fn update_background(
    mut last_bg_kind: ResMut<LastBackgroundKind>,
    bg_kind: Res<BackgroundKind>,
    mut commands: Commands,
    bg_root: Query<Entity, With<BackgroundRoot>>,
    asset_server: Res<AssetServer>,
) {
    let Ok(root_eid) = bg_root.get_single() else {
        error!("Weird stuff happening in updatebackground");
        return;
    };
    if last_bg_kind.0 == *bg_kind {
        return;
    }
    last_bg_kind.0 = *bg_kind;
    commands.entity(root_eid).despawn_descendants();
    commands
        .entity(root_eid)
        .with_children(|parent| match *bg_kind {
            BackgroundKind::None => (),
            BackgroundKind::ParallaxStars(num_stars) => {
                let depth_min = 5;
                let depth_max = 14;
                let scale_min = 1.0;
                let scale_max = 5.0;
                let mut rng = thread_rng();
                for _ in 0..num_stars {
                    let depth: u8 = rng.gen_range(depth_min..depth_max) as u8;
                    let frac_pos = Vec2 {
                        x: -0.5 + rng.gen::<f32>(),
                        y: -0.5 + rng.gen::<f32>(),
                    };
                    let scale = scale_min + rng.gen::<f32>() * (scale_max - scale_min);
                    let placement = PlacedBgBundle::basic_parallax(depth, frac_pos, scale);
                    let color = Color::hsla(rng.gen::<f32>() * 360.0, 0.8, 0.4, 1.0);
                    let sprite = SpriteBundle {
                        texture: asset_server.load("sprites/stars/7a.png"),
                        sprite: Sprite { color, ..default() },
                        ..default()
                    };
                    let sprite_l = SpriteBundle {
                        texture: asset_server.load("sprites/stars/7aL.png"),
                        ..default()
                    };
                    parent.spawn((placement.clone(), sprite, bg_sprite_layer()));
                    parent.spawn((placement, sprite_l, bg_light_layer()));
                }
            }
        });
}

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BackgroundKind::None);
        app.insert_resource(LastBackgroundKind(BackgroundKind::None));
        app.add_systems(Startup, setup_background);
        app.add_systems(FixedUpdate, move_bg_entities.after(camera_movement));
        app.add_systems(Update, update_background);
    }
}

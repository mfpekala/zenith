use std::collections::VecDeque;

use crate::{
    camera::{camera_movement, CameraMarker},
    drawing::{
        animation::{AnimationManager, MultiAnimationManager, SpriteInfo},
        effects::EffectVal,
        layering::{bg_light_layer_u8, bg_sprite_layer_u8},
    },
    math::Spleen,
    meta::{
        consts::{SCREEN_HEIGHT, SCREEN_WIDTH},
        game_state::{GameState, MetaState, SetMetaState, SetPaused},
    },
    physics::dyno::IntMoveable,
};
use bevy::prelude::*;
use rand::{thread_rng, Rng};

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum BgKind {
    None,
    ParallaxStars(usize),
}

#[derive(PartialEq, Clone, Debug)]
pub enum BgEffect {
    ScrollStars {
        dir: Vec2,
        length: f32,
        accel: bool,
        gs: Option<GameState>,
    },
}
impl BgEffect {
    /// The default menu star parallax scroll effect.
    /// Dir: 0.0 = stop, 1.0 = forward, -1.0 = backward
    pub fn default_menu_scroll(forward: bool, start: bool, ms: Option<MetaState>) -> Self {
        let dir = if forward { -1.0 } else { 1.0 } * Vec2::new(2.0, 0.2) * 4_010.4;
        BgEffect::ScrollStars {
            dir,
            length: 0.5,
            accel: start,
            gs: ms.map(|meta| GameState { meta, pause: None }),
        }
    }
}

#[derive(Resource)]
pub struct BgManager {
    current_kind: BgKind,
    next_kind: Option<BgKind>,
    current_effect: Option<(BgEffect, EffectVal)>,
    queued_effects: VecDeque<BgEffect>,
    last_cam_pos: IVec3,
}
impl BgManager {
    fn blank() -> Self {
        Self {
            current_kind: BgKind::None,
            next_kind: None,
            current_effect: None,
            queued_effects: VecDeque::new(),
            last_cam_pos: IVec3::default(),
        }
    }

    pub fn queue_effect(&mut self, effect: BgEffect) {
        self.queued_effects.push_back(effect)
    }

    /// Marks that the kind should change
    pub fn set_kind(&mut self, kind: BgKind) {
        self.next_kind = Some(kind);
    }

    /// Updates the current effect, returning (started_effect, finished_effect)
    fn update_effect(&mut self, time: &Res<Time>) -> (Option<BgEffect>, Option<BgEffect>) {
        let can_pop_front = match self.current_effect.as_mut() {
            Some((_effect, val)) => {
                val.timer.tick(time.delta());
                val.timer.finished() && !val.timer.just_finished()
            }
            None => true,
        };
        if can_pop_front {
            let finished_effect = self.current_effect.take().map(|thing| thing.0);
            let new_effect = match self.queued_effects.pop_front() {
                Some(effect) => {
                    let (time, spleen) = match &effect {
                        BgEffect::ScrollStars { length, accel, .. } => (
                            length,
                            if *accel {
                                Spleen::EaseInQuintic
                            } else {
                                Spleen::EaseOutQuintic
                            },
                        ),
                    };
                    self.current_effect =
                        Some((effect.clone(), EffectVal::new(0.0, 1.0, spleen, *time)));
                    Some(effect)
                }
                None => {
                    self.current_effect = None;
                    None
                }
            };
            return (new_effect, finished_effect);
        }
        (None, None)
    }

    pub fn has_active_effect(&self) -> bool {
        self.current_effect.is_some()
    }

    /// Grr hacky, is the current effect going to change the game state when it finished
    pub fn has_stateful_effect(&self) -> bool {
        match &self.current_effect {
            Some((effect, _)) => match effect {
                BgEffect::ScrollStars { gs, .. } => gs.is_some(),
            },
            None => false,
        }
    }

    pub fn clear_effects(&mut self) {
        self.current_effect = None;
        self.queued_effects.clear();
    }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize, Reflect)]
pub struct BgDepth {
    /// The logical depth of the entity. Effects position of the element.
    pub depth: u8,
    /// Should the entity shrink according to it's depth?
    pub shrink: bool,
    /// If set, the position of this element will wrap every `x` copies of SCREEN.
    /// I.e., if it is Some(1.0), then it will always be on screen, Some(2) will be on screen half the time, etc.
    pub wrap: Option<f32>,
    /// Internal state for moving according to depth
    depth_pos: Vec2,
    depth_bounds: Vec2,
}
impl BgDepth {
    /// Generate simple depth
    pub fn new(screen_pos: Vec2, depth: u8, shrink: bool, wrap: Option<f32>) -> Self {
        let mut result = Self {
            depth,
            shrink,
            wrap,
            ..default()
        };
        let og_screen_size = Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);
        let ref_screen_size =
            og_screen_size * result.dmult_translation() * result.wrap.unwrap_or(1.0);
        let fpos = Vec2 {
            x: screen_pos.x / SCREEN_WIDTH as f32,
            y: screen_pos.y / SCREEN_HEIGHT as f32,
        };
        result.depth_pos = Vec2::new(fpos.x * ref_screen_size.x, fpos.y * ref_screen_size.y);
        result.depth_bounds = ref_screen_size;
        result
    }

    /// Get screen pos from depth pos
    pub fn to_screen_pos(&self) -> Vec2 {
        let fpos = Vec2::new(
            self.depth_pos.x / self.depth_bounds.x,
            self.depth_pos.y / self.depth_bounds.y,
        );
        Vec2::new(fpos.x * SCREEN_WIDTH as f32, fpos.y * SCREEN_HEIGHT as f32)
    }

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

    /// Wraps the depth_pos if needed
    /// TODO: This is probably hella slow. Can remove some ops, or just make an int thing
    pub fn do_wrap(&mut self) {
        if let Some(wrap_num) = self.wrap {
            if self.depth_pos.x > self.depth_bounds.x * wrap_num * 0.5 {
                let diff = self.depth_pos.x - self.depth_bounds.x * wrap_num * 0.5;
                self.depth_pos.x = -self.depth_bounds.x * wrap_num * 0.5 + diff;
            }
            if self.depth_pos.x < -self.depth_bounds.x * wrap_num * 0.5 {
                let diff = -self.depth_pos.x + self.depth_bounds.x * wrap_num * 0.5;
                self.depth_pos.x = self.depth_bounds.x * wrap_num * 0.5 + diff;
            }
            if self.depth_pos.y > self.depth_bounds.y * wrap_num * 0.5 {
                let diff = self.depth_pos.y - self.depth_bounds.y * wrap_num * 0.5;
                self.depth_pos.y = -self.depth_bounds.y * wrap_num * 0.5 + diff;
            }
            if self.depth_pos.y < -self.depth_bounds.y * wrap_num * 0.5 {
                let diff = -self.depth_pos.y + self.depth_bounds.y * wrap_num * 0.5;
                self.depth_pos.y = self.depth_bounds.y * wrap_num * 0.5 + diff;
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize, Reflect)]
pub enum BgPlacementKind {
    #[default]
    Free,
    Parallax,
}

#[derive(Component, Clone, Debug, Default, serde::Serialize, serde::Deserialize, Reflect)]
#[reflect(Component, Serialize, Deserialize)]
/// Component driving the positioning of background elements, along with BgDepth
pub struct BgPlacement {
    pub screen_pos: Vec2,
    pub depth: BgDepth,
    pub vel: Vec2,
    pub kind: BgPlacementKind,
    pub base_scale: f32,
}
impl BgPlacement {
    pub fn from_screen_pos(
        screen_pos: Vec2,
        depth: u8,
        vel: Vec2,
        is_parallax: bool,
        base_scale: f32,
        wrap: Option<f32>,
    ) -> Self {
        Self {
            depth: BgDepth::new(screen_pos, depth, true, wrap),
            screen_pos,
            vel,
            kind: if is_parallax {
                BgPlacementKind::Parallax
            } else {
                BgPlacementKind::Free
            },
            base_scale,
        }
    }
}

#[derive(Bundle, Clone)]
pub(super) struct PlacedBgBundle {
    placement: BgPlacement,
    multi: MultiAnimationManager,
    spatial: SpatialBundle,
    name: Name,
}
impl PlacedBgBundle {
    /// Creates a bg bundle at the given screen pos
    /// This bundle will have pos-type Free and will NOT have a parallax effect
    pub(super) fn _basic_stationary(screen_pos: Vec2, depth: u8, scale: f32) -> Self {
        let placement =
            BgPlacement::from_screen_pos(screen_pos, depth, Vec2::ZERO, false, scale, None);
        let mut star = AnimationManager::single_static(SpriteInfo {
            path: "sprites/stars/7a.png".to_string(),
            size: UVec2::new(7, 7),
            ..default()
        });
        star.set_render_layers(vec![bg_sprite_layer_u8()]);
        let mut light = AnimationManager::single_static(SpriteInfo {
            path: "sprites/stars/7aL.png".to_string(),
            size: UVec2::new(7, 7),
            ..default()
        });
        light.set_render_layers(vec![bg_light_layer_u8()]);
        let multi = MultiAnimationManager::from_pairs(vec![("star", star), ("light", light)]);
        let spatial = SpatialBundle::default();

        Self {
            placement,
            multi,
            spatial,
            name: Name::new("bg_star"),
        }
    }

    /// Creates a bg bundle at the given screen pos
    /// This bundle will have pos-type Parallax and will have a parallax effect
    pub fn basic_parallax(screen_pos: Vec2, depth: u8, scale: f32) -> Self {
        let mut rng = thread_rng();
        let color = Color::hsla(rng.gen::<f32>() * 360.0, 0.8, 0.4, 1.0);
        let placement =
            BgPlacement::from_screen_pos(screen_pos, depth, Vec2::ZERO, true, scale, Some(2.0));
        let mut star = AnimationManager::single_static(SpriteInfo {
            path: "sprites/stars/7a.png".to_string(),
            size: UVec2::new(7, 7),
            color,
        });
        star.set_render_layers(vec![bg_sprite_layer_u8()]);
        let mut light = AnimationManager::single_static(SpriteInfo {
            path: "sprites/stars/7aL.png".to_string(),
            size: UVec2::new(7, 7),
            color,
        });
        light.set_render_layers(vec![bg_light_layer_u8()]);
        let multi = MultiAnimationManager::from_pairs(vec![("star", star), ("light", light)]);
        let spatial = SpatialBundle::default();

        Self {
            placement,
            multi,
            spatial,
            name: Name::new("bg_star"),
        }
    }
}

fn move_bg_entities(
    mut objects: Query<(&mut Transform, &mut BgPlacement)>,
    camera: Query<&IntMoveable, With<CameraMarker>>,
    mut bg_manager: ResMut<BgManager>,
) {
    let Ok(camera_mv) = camera.get_single() else {
        error!("Weird stuff in move bg entities with camera");
        return;
    };
    let cam_movement = (camera_mv.get_ipos() - bg_manager.last_cam_pos)
        .truncate()
        .as_vec2();
    bg_manager.last_cam_pos = camera_mv.get_ipos();
    for (mut tran, mut placement) in objects.iter_mut() {
        let vel = placement.vel.clone();
        placement.depth.depth_pos += vel;
        match placement.kind {
            // Careful, these are subtly different
            BgPlacementKind::Free => {}
            BgPlacementKind::Parallax => {
                placement.depth.depth_pos -= cam_movement;
                match &bg_manager.current_effect {
                    Some((BgEffect::ScrollStars { dir, accel, .. }, val)) => {
                        let mut val = val.interp();
                        if !accel {
                            val = 1.0 - val;
                        }
                        placement.depth.depth_pos += val * *dir;
                    }
                    None => (),
                }
            }
        }
        placement.depth.do_wrap();
        placement.screen_pos = placement.depth.to_screen_pos();
        tran.translation = placement
            .screen_pos
            .extend(-(placement.depth.depth as f32))
            .round();
        if placement.depth.shrink {
            tran.scale =
                (Vec2::ONE * placement.base_scale * placement.depth.dmult_scale()).extend(1.0);
        }
    }
}

#[derive(Component)]
pub struct BgRoot;

fn setup_bg(mut commands: Commands) {
    commands.spawn((Name::new("bg_root"), BgRoot, SpatialBundle::default()));
}

fn update_bg_manager(
    mut bg_manager: ResMut<BgManager>,
    mut commands: Commands,
    bg_root: Query<Entity, With<BgRoot>>,
    time: Res<Time>,
    mut meta_writer: EventWriter<SetMetaState>,
    mut pause_writer: EventWriter<SetPaused>,
) {
    let Ok(root_eid) = bg_root.get_single() else {
        error!("Weird stuff happening in update background");
        return;
    };

    // First handle the kind
    if let Some(next_kind) = bg_manager.next_kind.take() {
        if next_kind != bg_manager.current_kind {
            bg_manager.current_kind = next_kind.clone();
            commands.entity(root_eid).despawn_descendants();
            commands
                .entity(root_eid)
                .with_children(|parent| match next_kind {
                    BgKind::None => (),
                    BgKind::ParallaxStars(num_stars) => {
                        let depth_min = 5;
                        let depth_max = 14;
                        let scale_min = 1.0;
                        let scale_max = 5.0;
                        let mut rng = thread_rng();
                        for _ in 0..num_stars {
                            let depth: u8 = rng.gen_range(depth_min..depth_max) as u8;
                            let frac_pos = Vec2 {
                                x: -1.0 + rng.gen::<f32>() * 2.0,
                                y: -1.0 + rng.gen::<f32>() * 2.0,
                            };
                            let screen_pos = Vec2::new(
                                frac_pos.x * SCREEN_WIDTH as f32,
                                frac_pos.y * SCREEN_HEIGHT as f32,
                            );
                            let scale = scale_min + rng.gen::<f32>() * (scale_max - scale_min);
                            let placement =
                                PlacedBgBundle::basic_parallax(screen_pos, depth, scale);
                            parent.spawn(placement);
                        }
                    }
                });
        }
    }

    // Then handle the effect
    let (_started_effect, finished_effect) = bg_manager.update_effect(&time);
    match finished_effect {
        Some(BgEffect::ScrollStars { gs, .. }) => {
            if let Some(gs) = gs {
                meta_writer.send(SetMetaState(gs.meta.clone()));
                pause_writer.send(SetPaused(gs.pause));
            }
        }
        None => (),
    };
}

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BgManager::blank());
        app.register_type::<BgPlacement>();
        app.add_systems(Startup, setup_bg);
        app.add_systems(FixedUpdate, move_bg_entities.after(camera_movement));
        app.add_systems(Update, update_bg_manager);
    }
}

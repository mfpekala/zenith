use std::time::Duration;

use bevy::{prelude::*, render::view::RenderLayers};
use rand::{thread_rng, Rng};

use crate::{
    camera::CameraMarker,
    drawing::layering::sprite_layer,
    math::{lerp, lerp_color, Spleen},
    menu::paused::is_unpaused,
    meta::consts::FRAMERATE,
    physics::{should_apply_physics, BulletTime},
    ship::Ship,
};

#[derive(Component)]
pub struct ParticleLifespan {
    pub timer: Timer,
    /// NOTE: The _total_ amount of time the particle will be
    /// alive. CONSTANT.
    pub lifespan: f32,
}
impl ParticleLifespan {
    pub fn new(lifespan: f32) -> Self {
        let timer = Timer::new(Duration::from_secs_f32(lifespan), TimerMode::Once);
        Self { timer, lifespan }
    }

    pub fn tick(&mut self, delta: Duration) {
        self.timer.tick(delta);
    }

    /// The fraction of its lifespan that this particle has been alive for
    pub fn frac(&self) -> f32 {
        self.timer.fraction()
    }
}

#[derive(Component)]
pub struct ParticleBody {
    pub pos: Vec3,
    pub vel: Vec2,
    pub size: f32,
    pub color: Color,
}

#[derive(Component, Clone)]
pub struct ParticleSizing {
    pub spleen: Spleen,
}

#[derive(Component, Clone)]
pub struct ParticleColoring {
    pub end_color: Color,
    pub spleen: Spleen,
}
impl ParticleColoring {
    pub fn get_color(&self, x: f32, start_color: Color) -> Color {
        let x = self.spleen.interp(x);
        Color::Hsla {
            hue: lerp(x, start_color.h(), self.end_color.h()),
            saturation: lerp(x, start_color.s(), self.end_color.s()),
            lightness: lerp(x, start_color.l(), self.end_color.l()),
            alpha: lerp(x, start_color.a(), self.end_color.a()),
        }
    }
}

#[derive(Default, Clone)]
pub struct ParticleOptions {
    pub sizing: Option<ParticleSizing>,
    pub coloring: Option<ParticleColoring>,
}

#[derive(Bundle)]
pub struct ParticleBundle {
    pub lifespan: ParticleLifespan,
    pub body: ParticleBody,
    pub sprite: SpriteBundle,
    pub render_layers: RenderLayers,
}
impl ParticleBundle {
    fn new(body: ParticleBody, lifespan: f32) -> Self {
        Self {
            lifespan: ParticleLifespan::new(lifespan),
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: body.color,
                    ..default()
                },
                transform: Transform {
                    translation: body.pos,
                    scale: Vec3::ONE * body.size,
                    ..default()
                },
                ..default()
            },
            body,
            render_layers: sprite_layer(),
        }
    }

    pub fn spawn_options(
        commands: &mut Commands,
        body: ParticleBody,
        lifespan: f32,
        options: ParticleOptions,
    ) -> Entity {
        let id = commands.spawn(Self::new(body, lifespan)).id();
        if let Some(sizing) = options.sizing {
            commands.entity(id).insert(sizing);
        }
        if let Some(coloring) = options.coloring {
            commands.entity(id).insert(coloring);
        }
        id
    }
}

fn update_particles(
    mut commands: Commands,
    mut particles: Query<(
        Entity,
        &mut Transform,
        &mut ParticleBody,
        &mut ParticleLifespan,
    )>,
    time: Res<Time>,
    bullet_time: Res<BulletTime>,
) {
    for (id, mut tran, mut body, mut lifespan) in particles.iter_mut() {
        lifespan.tick(time.delta().mul_f32(bullet_time.factor()));
        if lifespan.timer.finished() {
            commands.entity(id).despawn_recursive();
            continue;
        }
        let vel = body.vel;
        body.pos += vel.extend(0.0);
        tran.translation = body.pos;
    }
}

fn size_particles(
    mut particles: Query<(
        &ParticleBody,
        &ParticleLifespan,
        &ParticleSizing,
        &mut Transform,
    )>,
) {
    for (body, lifespan, shrink, mut tran) in particles.iter_mut() {
        let new_size = body.size * (1.0 - shrink.spleen.interp(lifespan.frac()));
        tran.scale = Vec3::ONE * new_size;
    }
}

fn color_particles(
    mut particles: Query<(
        &ParticleBody,
        &ParticleLifespan,
        &ParticleColoring,
        &mut Sprite,
    )>,
) {
    for (body, lifespan, coloring, mut sprite) in particles.iter_mut() {
        let c = coloring.get_color(lifespan.frac(), body.color);
        sprite.color = c;
    }
}

#[derive(Component)]
pub struct ParticleSpawner {
    pub angle_range: (f32, f32),
    pub mag_range: (f32, f32),
    pub size_range: (f32, f32),
    pub color_range: (Color, Color),
    pub lifespan_range: (f32, f32),
    pub frequency_secs: f32,
    pub frequency_var: f32,
    pub timer: Timer,
    pub num_per_spawn: u32,
    pub segment: (Vec3, Vec3),
    pub options: ParticleOptions,
}
impl ParticleSpawner {
    pub fn default_ship_trail() -> Self {
        Self {
            angle_range: (0.0, 0.0),
            mag_range: (0.0, 0.0),
            size_range: (Ship::radius(), Ship::radius()),
            color_range: (Color::YELLOW, Color::YELLOW),
            lifespan_range: (0.5, 0.5),
            frequency_secs: 0.5 / FRAMERATE as f32,
            frequency_var: 0.0,
            timer: Timer::new(Duration::from_secs_f32(0.0), TimerMode::Once),
            num_per_spawn: 1,
            segment: (-Vec3::Z, -Vec3::Z),
            options: ParticleOptions {
                sizing: Some(ParticleSizing {
                    spleen: Spleen::EaseInQuad,
                }),
                coloring: Some(ParticleColoring {
                    end_color: Color::BLUE,
                    spleen: Spleen::EaseInQuad,
                }),
            },
        }
    }
}

#[derive(Bundle)]
pub struct ParticleSpawnerBundle {
    pub spawner: ParticleSpawner,
    pub spatial: SpatialBundle,
}

fn update_spawners(
    mut commands: Commands,
    mut spawners: Query<(&mut ParticleSpawner, &GlobalTransform)>,
    time: Res<Time>,
    bullet_time: Res<BulletTime>,
) {
    let mut rng = thread_rng();
    for (mut spawner, gtran) in spawners.iter_mut() {
        spawner
            .timer
            .tick(time.delta().mul_f32(bullet_time.factor()));
        if !spawner.timer.finished() {
            continue;
        }
        // First reset the timer
        spawner.timer = Timer::new(
            Duration::from_secs_f32(
                spawner.frequency_secs + rng.gen::<f32>() * spawner.frequency_var,
            ),
            TimerMode::Once,
        );
        // Then spawn the particles
        for _ in 0..spawner.num_per_spawn {
            let ang = lerp(rng.gen(), spawner.angle_range.0, spawner.angle_range.1);
            let mag = lerp(rng.gen(), spawner.mag_range.0, spawner.mag_range.1);
            let vel = Vec2 {
                x: ang.cos() * mag,
                y: ang.sin() * mag,
            };
            let size = lerp(rng.gen(), spawner.size_range.0, spawner.size_range.1);
            let color = lerp_color(rng.gen(), spawner.color_range.0, spawner.color_range.1);
            let pos = gtran.translation()
                + spawner.segment.0
                + rng.gen::<f32>() * (spawner.segment.1 - spawner.segment.0);
            let lifespan = lerp(
                rng.gen(),
                spawner.lifespan_range.0,
                spawner.lifespan_range.1,
            );
            ParticleBundle::spawn_options(
                &mut commands,
                ParticleBody {
                    pos,
                    vel,
                    size,
                    color,
                },
                lifespan,
                spawner.options.clone(),
            );
        }
    }
}

fn test_spawner(mut _commands: Commands) {
    // commands.spawn(ParticleSpawnerBundle {
    //     spatial: SpatialBundle::default(),
    //     spawner: ParticleSpawner::rainbow(),
    // });
}

pub fn register_particles(app: &mut App) {
    app.add_systems(Startup, test_spawner);
    app.add_systems(
        Update,
        (
            update_spawners,
            update_particles,
            size_particles,
            color_particles,
        )
            .run_if(is_unpaused),
    );
}

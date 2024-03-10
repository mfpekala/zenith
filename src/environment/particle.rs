use std::time::Duration;

use bevy::{prelude::*, render::view::RenderLayers, sprite::MaterialMesh2dBundle};
use rand::{thread_rng, Rng};

use crate::{
    camera::CameraMarker,
    drawing::{
        lightmap::{light_layer, sprite_layer},
        mesh::generate_new_mesh,
    },
    math::{lerp, lerp_color, regular_polygon, Spleen},
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

    pub fn is_dead(&self) -> bool {
        self.timer.finished()
    }

    /// The fraction of its lifespan that this particle has been alive for
    pub fn frac(&self) -> f32 {
        self.timer.elapsed_secs() / self.lifespan
    }
}

#[derive(Component)]
pub struct ParticleBody {
    pub pos: Vec2,
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

#[derive(Component, Clone)]
pub struct ParticleLighting {
    pub radius: f32,
    pub color: Color,
    pub spleen: Spleen,
}

#[derive(Bundle)]
pub struct ParticleLightingBundle {
    pub light: ParticleLighting,
    pub mesh: MaterialMesh2dBundle<ColorMaterial>,
    pub render_layers: RenderLayers,
}
impl ParticleLightingBundle {
    pub fn new(
        num_sides: u32,
        radius: f32,
        color: Color,
        spleen: Spleen,
        mats: &mut ResMut<Assets<ColorMaterial>>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> Self {
        let mat = mats.add(ColorMaterial::from(Color::Hsla {
            hue: color.h(),
            saturation: color.s(),
            lightness: color.l(),
            alpha: color.a(),
        }));
        let points = regular_polygon(num_sides, 0.0, radius);
        let mesh = generate_new_mesh(&points, &mat, meshes);
        Self {
            light: ParticleLighting {
                radius,
                color,
                spleen,
            },
            mesh,
            render_layers: light_layer(),
        }
    }
}

#[derive(Default, Clone)]
pub struct ParticleOptions {
    pub sizing: Option<ParticleSizing>,
    pub coloring: Option<ParticleColoring>,
    pub lighting: Option<ParticleLighting>,
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
                    translation: body.pos.extend(0.0),
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
        mats: &mut ResMut<Assets<ColorMaterial>>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> Entity {
        let id = commands.spawn(Self::new(body, lifespan)).id();
        if let Some(sizing) = options.sizing {
            commands.entity(id).insert(sizing);
        }
        if let Some(coloring) = options.coloring {
            commands.entity(id).insert(coloring);
        }
        if let Some(lighting) = options.lighting {
            commands.entity(id).with_children(|parent| {
                let lb = ParticleLightingBundle::new(
                    12,
                    lighting.radius,
                    lighting.color,
                    lighting.spleen,
                    mats,
                    meshes,
                );
                parent.spawn(lb);
            });
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
    cam: Query<&CameraMarker>,
) {
    let _cam = cam.single();
    for (id, mut tran, mut body, mut lifespan) in particles.iter_mut() {
        lifespan.tick(time.delta());
        if lifespan.is_dead() {
            commands.entity(id).despawn_recursive();
            continue;
        }
        let vel = body.vel;
        body.pos += vel;
        tran.translation = body.pos.extend(0.0);
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

fn light_particles(
    particles: Query<&ParticleLifespan>,
    mut lbs: Query<(&Parent, &ParticleLighting, &mut Transform)>,
) {
    for (parent, lighting, mut transform) in lbs.iter_mut() {
        let Ok(lifespan) = particles.get(parent.get()) else {
            continue;
        };
        transform.scale = Vec3::ONE * (1.0 - lighting.spleen.interp(lifespan.frac()));
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
    pub segment: (Vec2, Vec2),
    pub options: ParticleOptions,
}
impl ParticleSpawner {
    pub fn rainbow() -> Self {
        Self {
            angle_range: (0.0, 360.0),
            mag_range: (1.0, 10.0),
            size_range: (1.0, 25.0),
            color_range: (
                Color::Hsla {
                    hue: 0.0,
                    saturation: 1.0,
                    lightness: 0.6,
                    alpha: 1.0,
                },
                Color::Hsla {
                    hue: 360.0,
                    saturation: 1.0,
                    lightness: 0.6,
                    alpha: 1.0,
                },
            ),
            frequency_secs: 0.5,
            frequency_var: 0.5,
            timer: Timer::new(Duration::from_secs_f32(0.0), TimerMode::Once),
            lifespan_range: (0.5, 5.0),
            num_per_spawn: 5,
            segment: (Vec2::ONE * -100.0, Vec2::ONE * 100.0),
            options: ParticleOptions {
                lighting: Some(ParticleLighting {
                    radius: 10.0,
                    spleen: Spleen::EaseInQuad,
                    color: Color::WHITE,
                }),
                ..default()
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
    mut mats: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut rng = thread_rng();
    for (mut spawner, gtran) in spawners.iter_mut() {
        spawner.timer.tick(time.delta());
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
            let pos = gtran.translation().truncate()
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
                &mut mats,
                &mut meshes,
            );
        }
    }
}

fn setup_spawner(mut commands: Commands) {
    commands.spawn(ParticleSpawnerBundle {
        spatial: SpatialBundle::default(),
        spawner: ParticleSpawner::rainbow(),
    });
}

pub fn register_particles(app: &mut App) {
    app.add_systems(Startup, setup_spawner);
    app.add_systems(
        Update,
        (
            update_spawners,
            update_particles,
            size_particles,
            color_particles,
            light_particles,
        ),
    );
}

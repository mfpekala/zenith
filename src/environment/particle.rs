use std::time::Duration;

use bevy::{prelude::*, render::view::RenderLayers};
use rand::{thread_rng, Rng};

use crate::{
    camera::CameraMarker,
    drawing::lightmap::sprite_layer,
    math::{lerp, Spleen},
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

#[derive(Component)]
pub struct ParticleSizing {
    pub spleen: Spleen,
}

#[derive(Component)]
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

    pub fn spawn(commands: &mut Commands, body: ParticleBody, lifespan: f32) -> Entity {
        commands
            .spawn(Self::new(body, lifespan))
            .insert(ParticleSizing {
                spleen: Spleen::EaseInQuad,
            })
            .insert(ParticleColoring {
                spleen: Spleen::EaseInQuad,
                end_color: Color::BLUE,
            })
            .with_children(|_parent| {
                // TODO: Spawn light
            })
            .id()
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
    let cam = cam.single();
    for (id, mut tran, mut body, mut lifespan) in particles.iter_mut() {
        lifespan.tick(time.delta());
        if lifespan.is_dead() {
            commands.entity(id).despawn_recursive();
            continue;
        }
        let vel = body.vel;
        body.pos += vel;
        tran.translation = cam.pixel_align(body.pos).extend(0.0);
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

fn test_particles(mut commands: Commands, ship: Query<(&GlobalTransform, &Ship)>) {
    let Ok(ship) = ship.get_single() else {
        return;
    };
    let mut rng = thread_rng();
    if rng.gen_bool(0.9) {
        ParticleBundle::spawn(
            &mut commands,
            ParticleBody {
                pos: ship.0.translation().truncate(),
                vel: Vec2::ZERO,
                size: 15.0,
                color: Color::YELLOW,
            },
            0.5,
        );
    }
}

pub fn register_particles(app: &mut App) {
    app.add_systems(Update, test_particles);
    app.add_systems(Update, (update_particles, size_particles, color_particles));
}

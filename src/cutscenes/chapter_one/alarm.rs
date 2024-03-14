use crate::camera::CameraMarker;
use crate::camera::CameraMode;
use crate::cutscenes::clear_cutscene_entities;
use crate::cutscenes::ChapterOneCutscenes;
use crate::cutscenes::Cutscene;
use crate::cutscenes::CutsceneMarker;
use crate::cutscenes::StopCutscene;
use crate::drawing::animated::AnimatedNode;
use crate::drawing::animated::AnimationBundle;
use crate::drawing::animated::AnimationManager;
use crate::drawing::layering::bg_light_layer;
use crate::drawing::layering::bg_sprite_layer;
use crate::drawing::layering::light_layer;
use crate::drawing::layering::sprite_layer;
use crate::drawing::mesh::generate_new_screen_mesh;
use crate::drawing::sunrise_mat::SunriseMaterial;
use crate::environment::background::clear_background_entities;
use crate::environment::background::BgMarker;
use crate::environment::background::PlacedBgBundle;
use crate::is_in_cutscene;
use crate::math::lerp;
use crate::math::Spleen;
use crate::meta::consts::TuneableConsts;
use crate::when_cutscene_started;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::utils::hashbrown::HashMap;
use rand::thread_rng;
use rand::Rng;

const THIS_CUTSCENE: Cutscene = Cutscene::One(ChapterOneCutscenes::Alarm);

when_cutscene_started!(THIS_CUTSCENE, when_entered_alarm);
is_in_cutscene!(THIS_CUTSCENE, is_in_alarm);

const ALARM_CAM_HOME: IVec2 = IVec2 {
    x: -10_000,
    y: -10_000,
};

#[derive(Component)]
pub(super) struct AlarmCutsceneData {
    pub time: f32,
    pub sunrise_handle: Handle<SunriseMaterial>,
}

#[derive(Component)]
pub(super) struct WindowMarker;

#[derive(Component)]
pub(super) struct ClockMarker;

#[derive(Bundle)]
struct AlarmBgStarBundle {
    placement: PlacedBgBundle,
    sprite: SpriteBundle,
    layers: RenderLayers,
}

pub(super) fn setup_alarm_cutscene(
    mut commands: Commands,
    mut cam_q: Query<&mut CameraMarker>,
    tune: Res<TuneableConsts>,
    asset_server: Res<AssetServer>,
    bgs: Query<Entity, With<BgMarker>>,
    mut mats: ResMut<Assets<SunriseMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Clear the screen by removing background items and moving to the middle of nowhere
    // This lets us still use all the fancy lighting layers in cutscenes like this
    let mut cam = cam_q.single_mut();
    cam.mode = CameraMode::Controlled;
    cam.pos = ALARM_CAM_HOME;
    clear_background_entities(&mut commands, &bgs);

    // Add in the sunrise
    let sunrise_handle = mats.add(SunriseMaterial { time_frac: 0.0 });
    let mesh = MaterialMesh2dBundle {
        mesh: generate_new_screen_mesh(&mut meshes),
        material: sunrise_handle.clone(),
        transform: Transform::from_translation(ALARM_CAM_HOME.as_vec2().extend(0.0)),
        ..default()
    };
    commands.spawn((mesh, sprite_layer(), BgMarker));

    // Add in the stars
    let num_stars = tune.get_or("alarm_star_num_stars", 0.0) as i32;
    let depth_min = tune.get_or("alarm_star_depth_min", 0.0) as i32;
    let depth_max = (tune.get_or("alarm_star_depth_max", 1.0) as i32).max(depth_min + 1);
    let scale_min = tune.get_or("alarm_star_scale_min", 0.0);
    let scale_max = tune.get_or("alarm_star_scale_max", 1.0);
    let mut rng = thread_rng();
    for _ in 0..num_stars {
        let depth: u8 = rng.gen_range(depth_min..depth_max) as u8;
        let frac_pos = Vec2 {
            x: -0.5 + rng.gen::<f32>(),
            y: -0.5 + rng.gen::<f32>(),
        };
        let scale = scale_min + rng.gen::<f32>() * (scale_max - scale_min);
        let placement = PlacedBgBundle::basic_stationary(&tune, depth, frac_pos, scale);
        let sprite = SpriteBundle {
            texture: asset_server.load("sprites/stars/7a.png"),
            sprite: Sprite {
                color: Color::YELLOW,
                ..default()
            },
            ..default()
        };
        let sprite_l = SpriteBundle {
            texture: asset_server.load("sprites/stars/7aL.png"),
            ..default()
        };
        commands.spawn(AlarmBgStarBundle {
            placement: placement.clone(),
            sprite,
            layers: bg_sprite_layer(),
        });
        commands.spawn(AlarmBgStarBundle {
            placement,
            sprite: sprite_l,
            layers: bg_light_layer(),
        });
    }

    // Add in the window
    let initial_window_scale = tune.get_or("alarm_initial_window_scale", 0.0);
    let window_offset_y = tune.get_or("alarm_window_offset_y", 0.0);
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("sprites/cutscenes/alarm/window.png"),
            transform: Transform {
                scale: (Vec2::ONE * initial_window_scale).extend(-1.0),
                translation: (ALARM_CAM_HOME.as_vec2() + Vec2::new(0.0, window_offset_y))
                    .extend(1.0),
                ..default()
            },
            ..default()
        },
        sprite_layer(),
        CutsceneMarker,
        WindowMarker,
    ));

    // Add in the alarm clock
    let clock_node = AnimatedNode::from_path(
        &asset_server,
        &mut atlases,
        "sprites/cutscenes/alarm/clock.png",
        UVec2::new(40, 24),
        2,
        Some(24),
        None,
    );
    let clock_l_node = AnimatedNode::from_path(
        &asset_server,
        &mut atlases,
        "sprites/cutscenes/alarm/clockL.png",
        UVec2::new(40, 24),
        2,
        Some(24),
        None,
    );
    let mut clock_map = HashMap::new();
    clock_map.insert("clock".to_string(), clock_node);
    let clock_manager = AnimationManager::from_map(clock_map);
    let mut clock_l_map = HashMap::new();
    clock_l_map.insert("clock".to_string(), clock_l_node);
    let clock_l_manager = AnimationManager::from_map(clock_l_map);
    let clock_bundle = AnimationBundle::new("clock", clock_manager);
    let clock_l_bundle = AnimationBundle::new("clock", clock_l_manager);
    commands
        .spawn((
            ClockMarker,
            CutsceneMarker,
            SpatialBundle {
                transform: Transform::from_translation(
                    (ALARM_CAM_HOME.as_vec2() - Vec2::new(0.0, 36.0)).extend(2.0),
                ),
                visibility: Visibility::Hidden,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((clock_bundle, sprite_layer()));
            parent.spawn((clock_l_bundle, light_layer()));
        });

    // Add our data component
    commands.spawn((
        AlarmCutsceneData {
            time: 0.0,
            sunrise_handle,
        },
        CutsceneMarker,
    ));
}

pub(super) fn update_alarm_cutscene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut cutscene_data: Query<&mut AlarmCutsceneData>,
    mut mats: ResMut<Assets<SunriseMaterial>>,
    time: Res<Time>,
    tune: Res<TuneableConsts>,
    mut stars: Query<&mut Sprite, With<BgMarker>>,
    mut window: Query<&mut Transform, With<WindowMarker>>,
    mut clock: Query<&mut Visibility, With<ClockMarker>>,
) {
    let Ok(mut data) = cutscene_data.get_single_mut() else {
        return;
    };
    data.time += time.delta_seconds();

    // Get consts
    let alarm_sunrise_delay = tune.get_or("alarm_sunrise_delay", 0.0);
    let alarm_sunrise_length = tune.get_or("alarm_sunrise_length", 0.0);
    let alarm_window_delay = tune.get_or("alarm_window_delay", 0.0);
    let alarm_window_length = tune.get_or("alarm_window_length", 0.0);
    let alarm_alarm_delay = tune.get_or("alarm_alarm_delay", 0.0);

    // Lerp the various parts of the cutscene
    let sunrise_mat = mats.get_mut(data.sunrise_handle.id()).unwrap();
    sunrise_mat.time_frac = ((data.time - alarm_sunrise_delay) / alarm_sunrise_length)
        .max(0.0)
        .min(1.0);
    for mut star_sprite in stars.iter_mut() {
        let alpha_frac = ((data.time - alarm_sunrise_delay)
            / (alarm_sunrise_length + alarm_window_length / 2.0))
            .max(0.0)
            .min(1.0);
        let alpha_frac = 1.0 - alpha_frac;
        let color = Color::YELLOW;
        star_sprite.color = Color::hsla(
            color.h(),
            color.s() * (0.8 + 0.2 * alpha_frac),
            color.l() * (0.8 + 0.2 * alpha_frac),
            alpha_frac,
        );
    }
    let initial_window_scale = tune.get_or("alarm_initial_window_scale", 0.0);
    let final_window_scale = tune.get_or("alarm_final_window_scale", 0.0);
    let x = ((data.time - alarm_window_delay) / alarm_window_length)
        .max(0.0)
        .min(1.0);
    let x = Spleen::EaseOutQuad.interp(x);
    let window_scale = lerp(x, initial_window_scale, final_window_scale);
    if let Ok(mut window) = window.get_single_mut() {
        window.scale = (Vec2::ONE * window_scale).extend(1.0);
    }
    if alarm_alarm_delay < data.time {
        if let Ok(mut visibility) = clock.get_single_mut() {
            if *visibility != Visibility::Visible {
                *visibility = Visibility::Visible;
                commands.spawn((
                    AudioBundle {
                        source: asset_server.load("sound_effects/alarm.ogg"),
                        settings: PlaybackSettings {
                            mode: bevy::audio::PlaybackMode::Loop,
                            paused: false,
                            ..default()
                        },
                    },
                    CutsceneMarker,
                ));
            }
        }
    }
}

pub(super) fn stop_alarm_cutscene(
    mut cutscene: ResMut<Cutscene>,
    mut stop: EventReader<StopCutscene>,
    mut commands: Commands,
    bgs: Query<Entity, With<BgMarker>>,
    css: Query<Entity, With<CutsceneMarker>>,
) {
    let Some(_) = stop.read().last() else {
        return;
    };
    if *cutscene != THIS_CUTSCENE {
        return;
    }
    println!("Stopping cutscene");
    clear_background_entities(&mut commands, &bgs);
    clear_cutscene_entities(&mut commands, &css);
    *cutscene = Cutscene::None;
}

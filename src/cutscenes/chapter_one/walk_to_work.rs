use crate::{
    camera::{CameraMarker, CameraMode},
    cutscenes::{
        clear_cutscene_entities, ChapterOneCutscenes, Cutscene, CutsceneMarker,
        DurableCutsceneMarker, StartCutscene, StopCutscene,
    },
    drawing::{
        animated::{AnimatedNode, AnimationBundle},
        layering::{bg_sprite_layer, light_layer, sprite_layer},
        mesh::{generate_new_sprite_mesh, MeshOutline},
        sprite_mat::SpriteMaterial,
    },
    environment::background::{clear_background_entities, BgMarker},
    is_in_cutscene,
    meta::consts::{SCREEN_HEIGHT, SCREEN_WIDTH},
    physics::dyno::IntMoveable,
    when_cutscene_started,
};
use bevy::prelude::*;

const THIS_CUTSCENE: Cutscene = Cutscene::One(ChapterOneCutscenes::WalkToWork);

when_cutscene_started!(THIS_CUTSCENE, when_entered_walk_to_work);
is_in_cutscene!(THIS_CUTSCENE, is_in_walk_to_work);

const WALK2WORK_CAM_HOME: IVec2 = IVec2 {
    x: -11_000,
    y: -10_000,
};

pub(super) fn setup_walk_to_work_cutscene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut color_mats: ResMut<Assets<ColorMaterial>>,
    mut sprite_mats: ResMut<Assets<SpriteMaterial>>,
    mut cam_q: Query<(&mut IntMoveable, &mut CameraMarker)>,
    bgs: Query<Entity, With<BgMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    hot_walk: Res<Walk2WorkHot>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Clear the screen by removing background items and moving to the middle of nowhere
    // This lets us still use all the fancy lighting layers in cutscenes like this
    let (mut moveable, mut cam) = cam_q.single_mut();
    cam.mode = CameraMode::Controlled;
    moveable.pos = WALK2WORK_CAM_HOME.extend(0);
    clear_background_entities(&mut commands, &bgs);

    // Spawn in the grass
    let grass_handle = asset_server.load("sprites/cutscenes/walk2work/grass.png");
    let grass_pos_2d = WALK2WORK_CAM_HOME
        + IVec2::new(
            -(SCREEN_WIDTH as i32) / 2,
            -(SCREEN_HEIGHT as i32) / 2 + hot_walk.ground_height as i32,
        );
    commands.spawn((
        Name::new("grass_top"),
        SpriteBundle {
            texture: grass_handle,
            sprite: Sprite {
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..default()
            },
            transform: Transform::from_scale(Vec3::new(1.0, 0.333333, 1.0)),
            ..default()
        },
        CutsceneMarker,
        IntMoveable {
            pos: grass_pos_2d.extend(0),
            vel: Vec2::new(-hot_walk.ground_speed, 0.0),
            ..default()
        },
        sprite_layer(),
    ));
    commands.spawn((
        Name::new("grass_bot"),
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                anchor: bevy::sprite::Anchor::BottomLeft,
                ..default()
            },
            transform: Transform {
                scale: Vec3::new(SCREEN_WIDTH as f32, hot_walk.ground_height + 0.1, 1.0),
                translation: WALK2WORK_CAM_HOME.as_vec2().extend(0.0)
                    + Vec3::new(
                        -(SCREEN_WIDTH as f32) / 2.0,
                        -(SCREEN_HEIGHT as f32) / 2.0,
                        0.0,
                    ),
                ..default()
            },
            ..default()
        },
        CutsceneMarker,
        sprite_layer(),
    ));

    // Spawn in the person
    let walk_size = UVec2::new(24, 24);
    let walk_grow = hot_walk.walk_grow;
    let calc_walk_size = walk_size.as_vec2() * walk_grow;
    let walk_node = AnimatedNode::from_path(
        &asset_server,
        &mut atlases,
        "sprites/narf_limp.png",
        UVec2::new(24, 24),
        11,
        None,
        None,
    );
    let mut walk_bund = AnimationBundle::from_single_node("walk", walk_node);
    walk_bund.sprite_sheet.transform.scale = Vec3::ONE * walk_grow;
    commands.spawn((
        Name::new("narf-walking"),
        walk_bund,
        IntMoveable {
            pos: WALK2WORK_CAM_HOME.extend(0)
                + IVec3 {
                    x: -(SCREEN_WIDTH as i32) / 2 - 4,
                    y: -(SCREEN_HEIGHT as i32) / 2
                        + hot_walk.ground_height as i32
                        + hot_walk.walk_y_offset as i32
                        + (calc_walk_size.y / 2.0) as i32,
                    z: -1,
                },
            vel: Vec2::new(hot_walk.walk_speed, 0.0),
            ..default()
        },
        CutsceneMarker,
        sprite_layer(),
    ));

    // Spawn in the streetlamps
    let streetlamp_handle = asset_server.load("sprites/cutscenes/walk2work/streetlamp.png");
    let streetlamp_l_handle = asset_server.load("sprites/cutscenes/walk2work/streetlampL.png");
    for ix in 0..20 {
        let pos = WALK2WORK_CAM_HOME.extend(0)
            + IVec3::new(
                -160 + hot_walk.dist_between_streetlamps * ix,
                -(SCREEN_HEIGHT as i32) / 2
                    + hot_walk.ground_height as i32
                    + (hot_walk.walk_y_offset / 2.0) as i32,
                -2,
            );
        commands.spawn((
            Name::new(format!("streetlamp_{}", ix)),
            SpriteBundle {
                texture: streetlamp_handle.clone(),
                sprite: Sprite {
                    anchor: bevy::sprite::Anchor::BottomCenter,
                    ..default()
                },
                ..default()
            },
            CutsceneMarker,
            IntMoveable {
                pos,
                vel: Vec2::new(-hot_walk.ground_speed, 0.0),
                ..default()
            },
            sprite_layer(),
        ));
        commands.spawn((
            Name::new(format!("streetlampL_{}", ix)),
            SpriteBundle {
                texture: streetlamp_l_handle.clone(),
                sprite: Sprite {
                    anchor: bevy::sprite::Anchor::BottomCenter,
                    ..default()
                },
                transform: Transform::from_scale(Vec3::ONE * 2.0),
                ..default()
            },
            CutsceneMarker,
            IntMoveable {
                pos: pos + IVec3::new(0, -20, 0),
                vel: Vec2::new(-hot_walk.ground_speed, 0.0),
                ..default()
            },
            light_layer(),
        ));
    }

    // Spawn in the trash piles
    let trash_handle = asset_server.load("textures/trash.png");
    let trash_img_size = UVec2::new(160, 160);
    for (points, z) in hot_walk.trash_piles.iter() {
        let mut mins = IVec2::new(i32::MAX, i32::MAX);
        let mut maxs = IVec2::new(i32::MIN, i32::MIN);
        let mut fpoints = vec![];
        for (x, y) in points {
            let ivec = IVec2::new(*x, *y);
            mins = mins.min(ivec);
            maxs = maxs.max(ivec);
            fpoints.push(ivec.as_vec2());
        }
        let mesh_size = UVec2::new((maxs.x - mins.x) as u32, (maxs.y - mins.y) as u32);
        let (pos, size) = SpriteMaterial::random_sized_bounds(mesh_size, trash_img_size);
        let trash_mat = SpriteMaterial::from_handle(trash_handle.clone(), Some(pos), Some(size));
        let trash_handle = sprite_mats.add(trash_mat);
        let placed_pos = WALK2WORK_CAM_HOME.extend(*z)
            + IVec3::new(
                0,
                -(SCREEN_HEIGHT as i32) / 2 + hot_walk.ground_height as i32,
                0,
            );
        let mesh = generate_new_sprite_mesh(&fpoints, &trash_handle, &mut meshes);
        commands
            .spawn((
                mesh,
                CutsceneMarker,
                sprite_layer(),
                IntMoveable {
                    pos: placed_pos,
                    vel: Vec2::new(-hot_walk.trash_speed, 0.0),
                    ..default()
                },
            ))
            .with_children(|parent| {
                let outline = MeshOutline {
                    width: 2.0,
                    color: Color::BLACK,
                };
                let bund = outline.to_bundle(&fpoints, &mut color_mats, &mut meshes);
                parent.spawn((bund, sprite_layer()));
            });
    }

    commands.spawn((
        AudioBundle {
            source: asset_server.load("music/drunken_saloon.ogg"),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                paused: false,
                ..default()
            },
        },
        CutsceneMarker,
    ));

    let mut white_background = SpriteBundle::default();
    white_background.transform.scale = Vec3::ONE * 200.0;
    commands.spawn((
        Name::new("white_background"),
        white_background,
        bg_sprite_layer(),
    ));
}

pub(super) fn update_walk_to_work_cutscene() {}

pub(super) fn stop_walk_to_work_cutscene(
    cutscene: Res<Cutscene>,
    mut stop: EventReader<StopCutscene>,
    mut start: EventWriter<StartCutscene>,
    mut commands: Commands,
    bgs: Query<Entity, With<BgMarker>>,
    css: Query<Entity, With<CutsceneMarker>>,
    dcss: Query<(Entity, &DurableCutsceneMarker)>,
) {
    let Some(sdata) = stop.read().last() else {
        return;
    };
    if *cutscene != THIS_CUTSCENE {
        return;
    }
    clear_background_entities(&mut commands, &bgs);
    clear_cutscene_entities(&mut commands, sdata.0, &css, &dcss);

    start.send(StartCutscene(sdata.0));
}

#[derive(
    serde::Deserialize,
    bevy::asset::Asset,
    bevy::reflect::TypePath,
    Debug,
    PartialEq,
    Clone,
    Resource,
    Default,
)]
pub struct Walk2WorkHot {
    pub dummy: f32, // Nice to have to be able to save something useless to trigger reload
    pub ground_height: f32,
    pub ground_speed: f32,
    pub trash_piles: Vec<(Vec<(i32, i32)>, i32)>,
    pub trash_speed: f32,
    pub bird_lifetime: f32,
    pub walk_speed: f32,
    pub walk_grow: f32,
    pub walk_y_offset: f32,
    pub dist_between_streetlamps: i32,
}

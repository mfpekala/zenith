use crate::{
    camera::{CameraMarker, CameraMode},
    cutscenes::{
        clear_cutscene_entities, ChapterOneCutscenes, Cutscene, CutsceneMarker, StartCutscene,
        StopCutscene,
    },
    drawing::{
        layering::sprite_layer,
        mesh::{generate_new_sprite_mesh, MeshOutline, MeshOutlineBundle},
        sprite_mat::SpriteMaterial,
    },
    environment::background::{clear_background_entities, BgMarker},
    is_in_cutscene, when_cutscene_started,
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
    mut sprite_mats: ResMut<Assets<SpriteMaterial>>,
    mut color_mats: ResMut<Assets<ColorMaterial>>,
    mut cam_q: Query<&mut CameraMarker>,
    bgs: Query<Entity, With<BgMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    println!("Setting up walk to work");

    // Clear the screen by removing background items and moving to the middle of nowhere
    // This lets us still use all the fancy lighting layers in cutscenes like this
    let mut cam = cam_q.single_mut();
    cam.mode = CameraMode::Controlled;
    cam.pos = WALK2WORK_CAM_HOME;
    clear_background_entities(&mut commands, &bgs);

    let trash_img_handle = asset_server.load("textures/trash.png");
    let mesh_size = UVec2::new(30, 30);
    let image_size = UVec2::new(160, 160);
    for x in -2..2 {
        let (pos, size) = SpriteMaterial::random_sized_bounds(mesh_size, image_size);
        let trash_mat =
            SpriteMaterial::from_handle(trash_img_handle.clone(), Some(pos), Some(size));
        let trash_mat_handle = sprite_mats.add(trash_mat);
        let x = x * 40;
        let points = vec![
            Vec2::new(-(mesh_size.x as f32) / 2.0, -(mesh_size.y as f32) / 2.0)
                + Vec2::new(10.0, 20.0),
            Vec2::new(-(mesh_size.x as f32) / 2.0, (mesh_size.y as f32) / 2.0)
                + Vec2::new(10.0, 20.0),
            Vec2::new((mesh_size.x as f32) / 2.0, (mesh_size.y as f32) / 2.0)
                + Vec2::new(10.0, 20.0),
            Vec2::new((mesh_size.x as f32) / 2.0, -(mesh_size.y as f32) / 2.0)
                + Vec2::new(10.0, 20.0),
        ];
        let mut mesh = generate_new_sprite_mesh(&points, &trash_mat_handle, &mut meshes);
        mesh.transform.translation = (WALK2WORK_CAM_HOME + IVec2::new(x, 0))
            .as_vec2()
            .extend(10.0);
        commands
            .spawn((mesh, sprite_layer(), CutsceneMarker, Name::new("trash")))
            .with_children(|parent| {
                let outline = MeshOutline {
                    color: Color::BLACK,
                    width: 2.0,
                };
                let bund = MeshOutlineBundle::new(outline, &points, &mut color_mats, &mut meshes);
                parent.spawn((bund, sprite_layer()));
            });
    }
}

pub(super) fn update_walk_to_work_cutscene() {}

pub(super) fn stop_walk_to_work_cutscene(
    cutscene: Res<Cutscene>,
    mut stop: EventReader<StopCutscene>,
    mut start: EventWriter<StartCutscene>,
    mut commands: Commands,
    bgs: Query<Entity, With<BgMarker>>,
    css: Query<Entity, With<CutsceneMarker>>,
) {
    let Some(sdata) = stop.read().last() else {
        return;
    };
    if *cutscene != THIS_CUTSCENE {
        return;
    }
    println!("Clearing walk to work");

    clear_background_entities(&mut commands, &bgs);
    clear_cutscene_entities(&mut commands, &css);

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
    pub ground_height: f32,
    pub trash_piles: Vec<Vec<Vec2>>,
    pub bird_lifetime: f32,
}

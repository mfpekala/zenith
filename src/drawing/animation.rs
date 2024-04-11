use bevy::{
    prelude::*,
    render::{primitives::Aabb, view::RenderLayers},
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::hashbrown::{HashMap, HashSet},
};
use serde::{Deserialize, Serialize};

use crate::math::irect;

use super::{
    animation_mat::{AnimationMaterial, AnimationMaterialPlugin},
    mesh::{points_to_mesh, uvec2_bound},
};

#[derive(Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub struct SpriteInfo {
    pub path: String,
    pub size: UVec2,
}

#[derive(Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub struct AnimationNode {
    pub sprite: SpriteInfo,
    pub length: u32,
    pub next: Option<String>,
    pub pace: Option<u32>,
}

#[derive(Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub enum AnimationScale {
    #[default]
    Repeat,
    Grow,
}

#[derive(Component, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct AnimationManager {
    key: String,
    map: HashMap<String, AnimationNode>,
    points: Vec<IVec2>,
    scale: AnimationScale,
    offset: IVec3,
    angle: f32,
    is_changed: bool,
}
impl AnimationManager {
    pub fn change_key(&mut self, key: String) {
        self.key = key;
        self.is_changed = true;
    }

    pub fn single_static(sprite: SpriteInfo) -> Self {
        let node = AnimationNode {
            sprite: sprite.clone(),
            length: 1,
            ..default()
        };
        let mut map = HashMap::new();
        map.insert(sprite.path.clone(), node);
        Self {
            key: sprite.path.clone(),
            map,
            points: irect(sprite.size.x, sprite.size.y),
            ..default()
        }
    }
}
impl Default for AnimationManager {
    fn default() -> Self {
        Self {
            key: String::new(),
            map: HashMap::new(),
            points: vec![],
            scale: AnimationScale::Repeat,
            offset: IVec3::ZERO,
            angle: 0.0,
            is_changed: true,
        }
    }
}

#[derive(Component, Default)]
struct AnimationBody {
    handle_map: HashMap<String, Handle<Image>>,
}

/// Ix only increases once steps >= AnimationPace.0
#[derive(Component, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
struct AnimationIndex {
    ix: u32,
    steps: u32,
}

#[derive(Component, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
struct AnimationPace(u32);

#[derive(Component, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
struct AnimationLength(u32);

#[derive(Bundle, Default)]
struct AnimationBundle {
    body: AnimationBody,
    index: AnimationIndex,
    pace: AnimationPace,
    length: AnimationLength,
    render_layers: RenderLayers,
    mesh: Mesh2dHandle,
    material: Handle<AnimationMaterial>,
    spatial: SpatialBundle,
}

/// Looks for AnimationManagers that don't have AnimationBody and spawns them
fn materialize_animation_bodies(
    mut commands: Commands,
    managers: Query<(Entity, Option<&Children>), With<AnimationManager>>,
    bodies: Query<&AnimationBody>,
) {
    for (eid, children) in managers.iter() {
        let is_materialized = match children {
            None => false,
            Some(eids) => eids.iter().any(|eid| bodies.get(*eid).is_ok()),
        };
        if is_materialized {
            continue;
        }
        commands.entity(eid).with_children(|parent| {
            parent.spawn(AnimationBundle::default());
        });
    }
}

/// Looks for AnimationBodys whose manager has is_changed: true
/// NOTE: This does NOT progress animations, because this happens in Update, not FixedUpdate
/// It happens in Update so we can have these changes reflected as soon as possible
fn update_animation_bodies(
    mut commands: Commands,
    mut managers: Query<(&mut AnimationManager, &RenderLayers)>,
    mut bodies: Query<(
        Entity,
        &Parent,
        &mut AnimationBody,
        &mut AnimationPace,
        &mut AnimationLength,
        &mut Transform,
    )>,
    asset_server: Res<AssetServer>,
    mut mats: ResMut<Assets<AnimationMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (eid, parent, mut body, mut pace, mut length, mut tran) in bodies.iter_mut() {
        // See if we need to update
        let Ok((mut manager, render_layers)) = managers.get_mut(parent.get()) else {
            continue;
        };
        let Some(current_node) = manager.map.get(&manager.key) else {
            continue;
        };
        let current_node = current_node.clone();
        if !manager.is_changed {
            continue;
        }
        manager.is_changed = false;

        // Add the render layers
        commands.entity(eid).insert(render_layers.clone());

        // Update the body's handle map
        for animation_path in manager.map.keys() {
            if body.handle_map.get(animation_path).is_none() {
                let handle = asset_server.load(animation_path);
                body.handle_map.insert(animation_path.clone(), handle);
            }
        }
        let mut killing = HashSet::new();
        for animation_path in body.handle_map.keys() {
            if !manager.map.contains_key(animation_path) {
                killing.insert(animation_path.clone());
            }
        }
        for key in killing {
            body.handle_map.remove(&key);
        }

        // Set the pace
        pace.0 = current_node.pace.unwrap_or(2);

        length.0 = current_node.length;

        // Redo the mesh
        let image_handle = body.handle_map.get(&manager.key).unwrap().clone();
        let mat = AnimationMaterial::from_handle(image_handle, 0, Vec2::ZERO);
        let mat_ass = mats.add(mat);
        let fpoints: Vec<Vec2> = manager.points.iter().map(|p| p.as_vec2()).collect();
        let mesh = points_to_mesh(&fpoints, &mut meshes);
        let bund = MaterialMesh2dBundle {
            mesh,
            material: mat_ass,
            ..default()
        };
        // commands.entity(eid).insert(mat_ass);
        // commands.entity(eid).insert(mesh);
        commands.entity(eid).insert(bund);
        commands.entity(eid).remove::<Aabb>(); // Makes the engine recalculate the Aabb so culling is right

        // Update the translation
        tran.translation = manager.offset.as_vec3();
        tran.rotation = Quat::from_axis_angle(Vec3::Z, manager.angle);
    }
}

/// Actually play the animations. Happens during the FixedUpdate step.
fn play_animations(
    mut managers: Query<&mut AnimationManager>,
    mut bodies: Query<(
        &Parent,
        &Handle<AnimationMaterial>,
        &mut AnimationIndex,
        &AnimationPace,
        &AnimationLength,
    )>,
    mut mats: ResMut<Assets<AnimationMaterial>>,
) {
    for (parent, mat_handle, mut index, pace, length) in bodies.iter_mut() {
        let Ok(mut manager) = managers.get_mut(parent.get()) else {
            continue;
        };
        let Some(current_node) = manager.map.get(&manager.key) else {
            continue;
        };
        let Some(mat) = mats.get_mut(mat_handle.id()) else {
            continue;
        };
        // First update the material
        mat.index = index.ix as f32;
        mat.length = length.0 as f32;
        match manager.scale {
            AnimationScale::Repeat => {
                let fpoints = manager.points.iter().map(|p| p.as_vec2()).collect();
                let mesh_size = uvec2_bound(&fpoints);
                let image_size = current_node.sprite.size;
                mat.x_repetitions = mesh_size.x as f32 / image_size.x as f32;
                mat.y_repetitions = mesh_size.y as f32 / image_size.y as f32;
            }
            AnimationScale::Grow => {
                mat.x_repetitions = 1.0;
                mat.y_repetitions = 1.0;
            }
        }

        // Then progress the animation (so in case it swaps it'll be correct by next frame)
        let current_node = current_node.clone();
        index.steps += 1;
        if index.steps > pace.0 {
            index.ix += 1;
            index.steps = 0;
        }
        if index.ix > length.0 {
            index.ix = 0;
            if current_node.next.is_some() {
                manager.change_key(current_node.next.unwrap());
            }
        }
    }
}

pub struct GoatedAnimationPlugin;
impl Plugin for GoatedAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AnimationMaterialPlugin);
        app.add_systems(
            Update,
            (materialize_animation_bodies, update_animation_bodies).chain(),
        );
        app.add_systems(FixedUpdate, play_animations);

        app.register_type::<AnimationManager>();
        app.register_type::<AnimationIndex>();
        app.register_type::<AnimationLength>();
        app.register_type::<AnimationPace>();
    }
}

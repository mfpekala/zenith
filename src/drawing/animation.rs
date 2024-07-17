use bevy::{
    prelude::*,
    render::{primitives::Aabb, view::RenderLayers},
    sprite::Mesh2dHandle,
    utils::hashbrown::{HashMap, HashSet},
};
use serde::{Deserialize, Serialize};

use crate::math::irect;

use super::{
    animation_mat::{AnimationMaterial, AnimationMaterialPlugin},
    bordered_mesh::{materialize_bordered_meshes, update_bordered_meshes, BorderedMesh},
    layering::{bg_light_layer_u8, light_layer_u8, sprite_layer_u8},
    mesh::{ioutline_points, points_to_mesh, uvec2_bound},
};

#[derive(Clone, PartialEq, Reflect, Serialize, Deserialize, Debug)]
#[reflect(Serialize, Deserialize)]
pub struct SpriteInfo {
    pub path: String,
    pub size: UVec2,
    pub color: Color,
}
impl Default for SpriteInfo {
    fn default() -> Self {
        Self {
            path: "sprites/default.png".into(),
            size: UVec2::new(1, 1),
            color: Color::WHITE,
        }
    }
}

#[derive(Default, Clone, PartialEq, Reflect, Serialize, Deserialize, Debug)]
#[reflect(Serialize, Deserialize)]
pub struct AnimationNode {
    pub sprite: SpriteInfo,
    pub length: u32,
    pub next: Option<String>,
    pub pace: Option<u32>,
}

#[derive(Default, Clone, PartialEq, Reflect, Serialize, Deserialize, Debug)]
#[reflect(Serialize, Deserialize)]
pub enum AnimationScale {
    #[default]
    Repeat,
    Grow,
}

#[derive(Component, Clone, PartialEq, Reflect, Serialize, Deserialize, Debug)]
#[reflect(Component, Serialize, Deserialize)]
pub struct AnimationManager {
    key: String,
    pub map: HashMap<String, AnimationNode>,
    points: Vec<IVec2>,
    scale: AnimationScale,
    offset: IVec3,
    // Material rotation
    mat_rot: f32,
    // Rotation of the transform
    tran_angle: f32,
    render_layers_u8: Vec<u8>,
    hidden: bool,
    scroll: Vec2,
    is_changed: bool,
    force_index: Option<u32>,
    ephemeral: bool,
}
impl AnimationManager {
    pub fn current_node(&self) -> AnimationNode {
        self.map.get(&self.key).unwrap().clone()
    }

    pub fn get_key(&self) -> String {
        self.key.clone()
    }

    pub fn get_render_layers_u8(&self) -> Vec<u8> {
        self.render_layers_u8.clone()
    }

    pub fn set_key(&mut self, key: &str) {
        if key == &self.key {
            // Do nothing. Use reset_key if you want this to reset the sprite even if the key is the same
            return;
        }
        self.key = key.to_string();
        self.is_changed = true;
        self.force_index = Some(0);
    }

    pub fn reset_key(&mut self, key: &str) {
        self.key = key.to_string();
        self.is_changed = true;
        self.force_index = Some(0);
    }

    pub fn get_points(&self) -> Vec<IVec2> {
        self.points.clone()
    }

    pub fn set_points(&mut self, points: Vec<IVec2>) {
        if points == self.points {
            // Do nothing
            return;
        }
        self.points = points;
        self.is_changed = true;
    }

    pub fn set_mat_rot(&mut self, angle: f32) {
        if (self.mat_rot - angle).abs() < 0.001 {
            // Do nothing
            return;
        }
        self.mat_rot = angle;
        self.is_changed = true;
    }

    pub fn set_tran_angle(&mut self, angle: f32) {
        if (self.tran_angle - angle).abs() < 0.001 {
            // Do nothing
            return;
        }
        self.tran_angle = angle;
        self.is_changed = true;
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        if self.hidden == hidden {
            // Do nothing
            return;
        }
        self.hidden = hidden;
        self.is_changed = true;
    }

    pub fn set_render_layers(&mut self, render_layers_u8: Vec<u8>) {
        if render_layers_u8 == self.render_layers_u8 {
            // Do nothing
            return;
        }
        self.render_layers_u8 = render_layers_u8;
        self.is_changed = true;
    }

    pub fn set_offset(&mut self, offset: IVec3) {
        if self.offset == offset {
            // Do nothing
            return;
        }
        self.offset = offset;
        self.is_changed = true;
    }

    pub fn set_scroll(&mut self, scroll: Vec2) {
        if self.scroll.distance_squared(scroll) < 0.001 {
            // Do nothing
            return;
        }
        self.scroll = scroll;
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

    /// Creates a basic animation manager with multiple nodes. The initial node is the first one in the list
    pub fn from_static_pairs(pairs: Vec<(&str, SpriteInfo)>) -> Self {
        let mut map = HashMap::new();
        let first_key = pairs[0].0.to_string();
        for (key, sprite) in pairs {
            let node = AnimationNode {
                sprite,
                length: 1,
                ..default()
            };
            map.insert(key.to_string(), node);
        }
        let first = map.get(&first_key).unwrap();
        let points = irect(first.sprite.size.x, first.sprite.size.y);
        Self {
            key: first_key,
            map,
            points,
            ..default()
        }
    }

    pub fn single_repeating(sprite: SpriteInfo, length: u32) -> Self {
        let node = AnimationNode {
            sprite: sprite.clone(),
            length,
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

    /// Creates an animation manager with the provided nodes and labels.
    /// The first node in the list will be the initial animation
    pub fn from_nodes(node_info: Vec<(&str, AnimationNode)>) -> Self {
        let mut map = HashMap::new();
        let first = node_info[0].clone();
        for (key, node) in node_info {
            map.insert(key.to_string(), node);
        }
        let points = irect(first.1.sprite.size.x, first.1.sprite.size.y);
        Self {
            key: first.0.to_string(),
            map,
            points,
            ..default()
        }
    }

    /// Forces the animation_manager to be ephemeral
    pub fn force_ephemeral(mut self) -> Self {
        self.ephemeral = true;
        self
    }

    /// Forces the animation_manager to take a single render layer
    pub fn force_render_layer(mut self, render_layer: u8) -> Self {
        self.render_layers_u8 = vec![render_layer];
        self
    }

    /// Forces the animation_manager to take a mat_rot
    pub fn force_mat_rot(mut self, angle: f32) -> Self {
        self.mat_rot = angle;
        self
    }

    /// Forces the animation_manager to take points
    pub fn force_points(mut self, points: Vec<IVec2>) -> Self {
        self.points = points;
        self
    }

    /// Forces the animation_manager to take offset
    pub fn force_offset(mut self, offset: IVec3) -> Self {
        self.offset = offset;
        self
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
            mat_rot: 0.0,
            tran_angle: 0.0,
            render_layers_u8: vec![sprite_layer_u8()],
            hidden: false,
            scroll: Vec2::ZERO,
            is_changed: true,
            force_index: Some(0),
            ephemeral: false,
        }
    }
}

#[derive(Component, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
struct MultiBodyMarker(String);

#[derive(Component, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct MultiAnimationManager {
    pub map: HashMap<String, AnimationManager>,
    pub is_coup: bool,
}
impl MultiAnimationManager {
    pub fn from_pairs(pairs: Vec<(&str, AnimationManager)>) -> Self {
        let mut map = HashMap::new();
        for (key, manager) in pairs {
            map.insert(key.to_string(), manager);
        }
        Self {
            map,
            is_coup: false,
        }
    }

    /// For each animation node in a lightable layer (bg or sprite)
    /// will add a light of the same size in the corresponding light layer
    pub fn well_lit(anim: AnimationManager) -> Self {
        let mut lighting_layers = vec![];
        for layer in anim.get_render_layers_u8() {
            if layer == sprite_layer_u8() {
                lighting_layers.push(light_layer_u8());
            }
            if layer == bg_light_layer_u8() {
                lighting_layers.push(bg_light_layer_u8());
            }
        }
        let mut light = anim.clone();
        for (_key, node) in light.map.iter_mut() {
            node.sprite.path = SpriteInfo::default().path;
            node.sprite.color = SpriteInfo::default().color;
        }
        light.set_render_layers(lighting_layers);

        let mut map = HashMap::new();
        map.insert("core".to_string(), anim);
        map.insert("light".to_string(), light);

        Self {
            map,
            is_coup: false,
        }
    }

    /// Makes a bordered mesh
    pub fn bordered_mesh(
        points: Vec<IVec2>,
        inner: SpriteInfo,
        outer: SpriteInfo,
        width: f32,
    ) -> Self {
        let inner =
            AnimationManager::single_static(inner).force_points(ioutline_points(&points, -width));
        let outer = AnimationManager::single_static(outer)
            .force_points(points.clone())
            .force_offset(-IVec3::Z);
        let mut map = HashMap::new();
        map.insert("inner".to_string(), inner);
        map.insert("outer".to_string(), outer);

        Self {
            map,
            is_coup: false,
        }
    }
}

/// Marks an animation body as not needing to be queried in `play_animations`
#[derive(Component)]
struct AnimationStatic;

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

#[derive(Component, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
struct AnimationScroll {
    vec: Vec2,
    reset: bool,
}

#[derive(Bundle, Default)]
struct AnimationBundle {
    name: Name,
    body: AnimationBody,
    index: AnimationIndex,
    pace: AnimationPace,
    length: AnimationLength,
    scroll: AnimationScroll,
    render_layers: RenderLayers,
    mesh: Mesh2dHandle,
    material: Handle<AnimationMaterial>,
    spatial: SpatialBundle,
}

/// Looks for AnimationManagers that don't have AnimationBody and spawns them
fn materialize_animation_bodies(
    mut commands: Commands,
    mut managers: Query<
        (Entity, Option<&Children>, &mut AnimationManager),
        Without<MultiAnimationManager>,
    >,
    mut multis: Query<
        (Entity, Option<&Children>, &mut MultiAnimationManager),
        Without<AnimationManager>,
    >,
    bodies: Query<Option<&MultiBodyMarker>, With<AnimationBody>>,
) {
    // Handle the single cases
    for (eid, children, mut manager) in managers.iter_mut() {
        let is_materialized = match children {
            None => false,
            Some(eids) => eids.iter().any(|eid| bodies.get(*eid).is_ok()),
        };
        if is_materialized {
            continue;
        }
        commands.entity(eid).with_children(|parent| {
            parent.spawn(AnimationBundle {
                name: Name::new("AnimationBody"),
                ..default()
            });
        });
        manager.is_changed = true;
    }
    // Handle the multi cases
    for (eid, children, mut multi) in multis.iter_mut() {
        for (key, manager) in multi.map.iter_mut() {
            let is_materialized = match children {
                None => false,
                Some(eids) => eids.iter().any(|eid| {
                    // Yoo
                    match bodies.get(*eid) {
                        Ok(Some(bm)) => &bm.0 == key,
                        _ => false,
                    }
                }),
            };
            if is_materialized {
                continue;
            }
            commands.entity(eid).with_children(|parent| {
                parent.spawn((
                    AnimationBundle {
                        name: Name::new("AnimationBody"),
                        ..default()
                    },
                    MultiBodyMarker(key.clone()),
                ));
            });
            manager.is_changed = true;
        }
    }
}

/// A kind of overkill way to prevent ABA problems in the map. Basically whenever such
/// issues could occur, just delete all saved handles and mark is_changed so things work out
fn resolve_animation_coups(
    mut multis: Query<(&mut MultiAnimationManager, &Children)>,
    mut bodies: Query<(
        &mut AnimationBody,
        &MultiBodyMarker,
        &Handle<AnimationMaterial>,
    )>,
    mut mats: ResMut<Assets<AnimationMaterial>>,
) {
    for (mut multi, children) in multis.iter_mut() {
        if !multi.is_coup {
            continue;
        }
        for child in children.iter() {
            let Ok((mut body, multi_marker, mat)) = bodies.get_mut(*child) else {
                continue;
            };
            mats.remove(mat.id());
            body.handle_map = HashMap::new();
            let Some(manager) = multi.map.get_mut(&multi_marker.0) else {
                continue;
            };
            manager.is_changed = true;
        }
        multi.is_coup = false;
    }
}

/// Looks for AnimationBodys whose manager has is_changed: true
/// NOTE: This does NOT progress animations, because this happens in Update, not FixedUpdate
/// It happens in Update so we can have these changes reflected as soon as possible
fn update_animation_bodies(
    mut commands: Commands,
    mut managers: Query<&mut AnimationManager, Without<MultiAnimationManager>>,
    mut multis: Query<&mut MultiAnimationManager, Without<AnimationManager>>,
    mut bodies: Query<(
        Entity,
        &Parent,
        &mut AnimationBody,
        &mut AnimationPace,
        &mut AnimationLength,
        &mut AnimationScroll,
        &mut Transform,
        &mut Visibility,
        Option<&MultiBodyMarker>,
    )>,
    asset_server: Res<AssetServer>,
    mut mats: ResMut<Assets<AnimationMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (
        eid,
        parent,
        mut body,
        mut pace,
        mut length,
        mut scroll,
        mut tran,
        mut vis,
        multi_marker,
    ) in bodies.iter_mut()
    {
        // See if we need to update
        let (manager, current_node) = match multi_marker {
            Some(mbm) => {
                let Ok(mut multi) = multis.get_mut(parent.get()) else {
                    continue;
                };
                let Some(manager) = multi.map.get_mut(&mbm.0) else {
                    continue;
                };
                if !manager.is_changed {
                    continue;
                }
                manager.is_changed = false;
                let crystal_manager = manager.clone();
                let current_node = manager.current_node();
                (crystal_manager, current_node)
            }
            None => {
                let Ok(mut manager) = managers.get_mut(parent.get()) else {
                    continue;
                };
                if !manager.is_changed {
                    continue;
                }
                manager.is_changed = false;
                let crystal_manager = manager.clone();
                let current_node = manager.current_node();
                (crystal_manager, current_node)
            }
        };

        // Add the render layers
        let render_layers = RenderLayers::from_layers(&manager.render_layers_u8);
        commands.entity(eid).insert(render_layers);

        // Update the body's handle map
        for (key, node) in manager.map.iter() {
            if body.handle_map.get(key).is_none() {
                let handle = asset_server.load(&node.sprite.path);
                body.handle_map.insert(key.clone(), handle);
            }
        }
        let mut killing = HashSet::new();
        for key in body.handle_map.keys() {
            if !manager.map.contains_key(key) {
                killing.insert(key.clone());
            }
        }
        for key in killing {
            body.handle_map.remove(&key);
        }

        // Set the pace
        pace.0 = current_node.pace.unwrap_or(1);

        // Set the length
        length.0 = current_node.length;

        // Set the scroll
        scroll.vec = manager.scroll;
        scroll.reset = true;

        if length.0 <= 1 && scroll.vec.length_squared() < 0.00001 {
            commands.entity(eid).insert(AnimationStatic);
        } else {
            commands.entity(eid).remove::<AnimationStatic>();
        }

        // Redo the mesh
        let fpoints: Vec<Vec2> = manager.points.iter().map(|p| p.as_vec2()).collect();
        let mesh_size = uvec2_bound(&fpoints);
        let x_rep = mesh_size.x as f32 / current_node.sprite.size.x as f32;
        let y_rep = mesh_size.y as f32 / current_node.sprite.size.y as f32;
        let image_handle = body.handle_map.get(&manager.key).unwrap().clone();
        let mut mat = AnimationMaterial::from_handle(
            image_handle,
            length.0,
            Vec2::new(x_rep, y_rep),
            current_node.sprite.color,
        );
        mat.rot = manager.mat_rot;
        mat.ephemeral = manager.ephemeral;
        let mat_ass = mats.add(mat);
        let mesh = points_to_mesh(&fpoints, &mut meshes);
        commands.entity(eid).insert(mat_ass);
        commands.entity(eid).insert(mesh);
        commands.entity(eid).remove::<Aabb>(); // Makes the engine recalculate the Aabb so culling is right

        // Update the translation/visibility
        *vis = if manager.hidden {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        tran.translation = manager.offset.as_vec3();
        tran.rotation = Quat::from_axis_angle(Vec3::Z, manager.tran_angle);
    }
}

/// For bodies who's parent is a MultiAnimationBundle, check if the key doesn't exist, and if it doesn't, despawn
fn dematerialize_animation_bodies(
    mut commands: Commands,
    multis: Query<(Entity, &MultiAnimationManager)>,
    bodies: Query<(Entity, &Parent, &MultiBodyMarker)>,
) {
    for (eid, parent, multi_marker) in bodies.iter() {
        let Ok((pid, multi)) = multis.get(parent.get()) else {
            // Uh oh
            continue;
        };
        if multi.map.contains_key(&multi_marker.0) {
            continue;
        }
        commands.entity(pid).remove_children(&[eid]);
        commands.entity(eid).despawn_recursive();
    }
}

/// Actually play the animations. Happens during the FixedUpdate step.
fn play_animations(
    mut managers: Query<&mut AnimationManager>,
    mut multis: Query<&mut MultiAnimationManager>,
    mut bodies: Query<
        (
            &Parent,
            &Handle<AnimationMaterial>,
            &mut AnimationIndex,
            &AnimationPace,
            &AnimationLength,
            &mut AnimationScroll,
            Option<&MultiBodyMarker>,
        ),
        Without<AnimationStatic>,
    >,
    mut mats: ResMut<Assets<AnimationMaterial>>,
) {
    for (parent, mat_handle, mut index, pace, length, mut scroll, multi_marker) in bodies.iter_mut()
    {
        let mut shared_logic = |manager: &mut AnimationManager| {
            let current_node = manager.current_node();
            let Some(mat) = mats.get_mut(mat_handle.id()) else {
                return;
            };
            // Zeroth, update the index if needed
            if let Some(forced) = manager.force_index {
                index.ix = forced;
                index.steps = 0;
                manager.force_index = None;
            }
            // First update the material
            mat.index = index.ix as f32;
            mat.length = length.0 as f32;
            mat.x_offset += scroll.vec.x / 3.0;
            mat.x_offset = mat.x_offset.rem_euclid(1.0);
            mat.y_offset += scroll.vec.y / 3.0;
            mat.y_offset = mat.y_offset.rem_euclid(1.0);
            if scroll.reset {
                mat.x_offset = 0.0;
                mat.y_offset = 0.0;
                scroll.reset = false;
            }
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
            if index.ix >= length.0 {
                index.ix = 0;
                if current_node.next.is_some() {
                    manager.set_key(&current_node.next.unwrap());
                }
            }
        };
        match multi_marker {
            Some(mbm) => {
                let Ok(mut multi) = multis.get_mut(parent.get()) else {
                    continue;
                };
                let Some(manager) = multi.map.get_mut(&mbm.0) else {
                    continue;
                };
                shared_logic(manager);
            }
            None => {
                let Ok(mut manager) = managers.get_mut(parent.get()) else {
                    continue;
                };
                shared_logic(&mut manager);
            }
        };
    }
}

pub struct GoatedAnimationPlugin;
impl Plugin for GoatedAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AnimationMaterialPlugin);
        app.add_systems(
            Update,
            (
                materialize_animation_bodies,
                resolve_animation_coups,
                // update_animation_bodies,
                dematerialize_animation_bodies,
            )
                .chain(),
        );
        app.add_systems(
            Update,
            (materialize_bordered_meshes, update_bordered_meshes).chain(),
        );
        app.add_systems(
            FixedUpdate,
            (update_animation_bodies, play_animations).chain(),
        );

        app.register_type::<AnimationManager>();
        app.register_type::<MultiAnimationManager>();
        app.register_type::<MultiBodyMarker>();
        app.register_type::<AnimationIndex>();
        app.register_type::<AnimationPace>();
        app.register_type::<AnimationLength>();
        app.register_type::<AnimationScroll>();
        app.register_type::<BorderedMesh>();
        app.register_type::<AnimationMaterial>();
        app.register_asset_reflect::<AnimationMaterial>();
    }
}

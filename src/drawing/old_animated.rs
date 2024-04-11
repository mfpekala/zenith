use bevy::{prelude::*, render::view::RenderLayers, utils::hashbrown::HashMap};
use serde::{Deserialize, Serialize};

use crate::{
    editor::save::SaveMarker,
    uid::{fresh_uid, UId, UIdMarker},
};

/// If you don't need to spawn it right away, you can spawn this stub
/// in an AnimatedManagerStub and a system will clean it up to spawn
#[derive(Component, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct AnimatedNodeStub {
    pub path: String,
    pub size: UVec2,
    pub length: u8,
    pub next: Option<String>,
    pub pace: Option<u8>,
}

/// Information about a specific animation state that an object can be in
#[derive(Debug, Clone, Component)]
pub struct AnimatedNode {
    pub handle: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub pace: Option<u8>,
    length: u8,
    pub next: Option<String>,
}
impl AnimatedNode {
    pub fn from_path(
        asset_server: &Res<AssetServer>,
        atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
        path: &str,
        size: UVec2,
        length: u8,
        pace: Option<u8>,
        next: Option<String>,
    ) -> Self {
        let handle = asset_server.load(path.to_string());
        let layout = atlases.add(TextureAtlasLayout::from_grid(
            size.as_vec2(),
            length as usize,
            1,
            None,
            None,
        ));
        Self {
            handle,
            layout,
            length,
            pace,
            next,
        }
    }
}

#[derive(Component, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct AnimationManagerStub {
    pub map: HashMap<String, AnimatedNodeStub>,
}
impl AnimationManagerStub {
    pub fn single_repeating(
        key: &str,
        path: &str,
        size: UVec2,
        length: u8,
        pace: Option<u8>,
    ) -> Self {
        let mut map = HashMap::new();
        map.insert(
            key.to_string(),
            AnimatedNodeStub {
                path: path.to_string(),
                size,
                length,
                next: None,
                pace,
            },
        );
        Self { map }
    }

    pub fn unstub(
        &self,
        asset_server: &Res<AssetServer>,
        atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
    ) -> AnimationManager {
        let mut map = HashMap::new();
        for (key, val) in self.map.iter() {
            map.insert(
                key.clone(),
                AnimatedNode::from_path(
                    asset_server,
                    atlases,
                    &val.path,
                    val.size,
                    val.length,
                    val.pace,
                    val.next.clone(),
                ),
            );
        }
        AnimationManager::from_map(map)
    }
}

#[derive(Component, Debug, Clone)]
pub struct AnimationManager {
    pub map: HashMap<String, AnimatedNode>,
    pub idx: u8,
    pub offset: u8,
    pub flip_x: bool,
    pub flip_y: bool,
    pub paused: bool,
}
impl AnimationManager {
    pub fn single_repeating(
        key: &str,
        path: &str,
        size: UVec2,
        length: u8,
        pace: Option<u8>,
        asset_server: &Res<AssetServer>,
        atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
    ) -> Self {
        let mut map = HashMap::new();
        map.insert(
            key.to_string(),
            AnimatedNode::from_path(asset_server, atlases, path, size, length, pace, None),
        );
        Self::from_map(map)
    }

    pub fn from_map(map: HashMap<String, AnimatedNode>) -> Self {
        Self {
            map,
            idx: 0,
            offset: 0,
            flip_x: false,
            flip_y: false,
            paused: false,
        }
    }
}

#[derive(Component)]
pub struct AnimationKey(pub String);

#[derive(Component)]
pub struct AnimationHeadStub {
    pub uid: UId,
    pub head: AnimationHead,
}
impl AnimationHeadStub {
    pub fn from_single_stub(stub: AnimationStub) -> Self {
        Self {
            uid: fresh_uid(),
            head: AnimationHead { stubs: vec![stub] },
        }
    }

    pub fn from_stubs(stubs: Vec<AnimationStub>) -> Self {
        Self {
            uid: fresh_uid(),
            head: AnimationHead { stubs },
        }
    }
}

#[derive(Component, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct AnimationHead {
    pub stubs: Vec<AnimationStub>,
}

#[derive(Component, Default, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct AnimationStub {
    pub uid: UId,
    pub manager: AnimationManagerStub,
    pub key: String,
    pub render_layers: Vec<u8>,
    pub offset: Vec3,
}
impl AnimationStub {
    /// Basically just a sprite. Abusing notation to reuse this hehe.
    pub fn single_static(path: &str, size: UVec2, render_layer: u8) -> Self {
        Self {
            uid: fresh_uid(),
            manager: AnimationManagerStub::single_repeating(path, path, size, 1, None),
            key: path.to_string(),
            render_layers: vec![render_layer],
            offset: Vec3::ZERO,
        }
    }

    /// An animation stub containing a single repeating animation
    pub fn single_repeating(
        key: &str,
        path: &str,
        size: UVec2,
        length: u8,
        pace: Option<u8>,
        render_layer: u8,
    ) -> Self {
        Self {
            uid: fresh_uid(),
            manager: AnimationManagerStub::single_repeating(key, path, size, length, pace),
            key: key.to_string(),
            render_layers: vec![render_layer],
            offset: Vec3::ZERO,
        }
    }
}

#[derive(Component)]
pub struct AnimationStubs(pub Vec<AnimationStub>);

#[derive(Bundle)]
struct AnimationBundle {
    pub uid: UIdMarker,
    pub sprite_sheet: SpriteSheetBundle,
    pub manager: AnimationManager,
    pub val: AnimationKey,
}

pub(super) fn resolve_animation_head_stubs(
    mut commands: Commands,
    stubs: Query<(Entity, &AnimationHeadStub)>,
) {
    for (eid, stub) in stubs.iter() {
        commands.entity(eid).with_children(|parent| {
            parent.spawn((
                UIdMarker(stub.uid),
                AnimationHead {
                    stubs: stub.head.stubs.clone(),
                },
                SpatialBundle::default(),
                SaveMarker,
            ));
        });
        commands.entity(eid).remove::<AnimationHeadStub>();
    }
}

pub(super) fn update_animation_heads(
    mut commands: Commands,
    heads: Query<(Entity, &AnimationHead, Option<&Children>)>,
    managers: Query<Entity>,
) {
    for (eid, head, children) in heads.iter() {
        match children {
            None => {
                commands
                    .entity(eid)
                    .insert(AnimationStubs(head.stubs.clone()));
            }
            Some(children) => {
                for child in children {
                    let Ok(_) = managers.get(*child) else {
                        commands.entity(eid).remove::<Children>();
                        continue;
                    };
                }
                // TODO: Update animation to follow head
            }
        }
    }
}

pub(super) fn materialize_animation_stubs(
    mut commands: Commands,
    stubs_q: Query<(Entity, &AnimationStubs)>,
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    for (id, stubs) in stubs_q.iter() {
        for stub in stubs.0.iter() {
            let full_manager = stub.manager.unstub(&asset_server, &mut atlases);
            let initial_node = full_manager.map.get(&stub.key).unwrap();
            let sprite_sheet = SpriteSheetBundle {
                texture: initial_node.handle.clone(),
                atlas: TextureAtlas {
                    layout: initial_node.layout.clone(),
                    index: 0,
                },
                transform: Transform::from_translation(stub.offset),
                ..default()
            };
            commands.entity(id).with_children(|parent| {
                parent.spawn((
                    full_manager,
                    AnimationKey(stub.key.clone()),
                    sprite_sheet,
                    UIdMarker(stub.uid),
                    RenderLayers::from_layers(&stub.render_layers),
                ));
            });
        }
        commands.entity(id).remove::<AnimationStubs>();
    }
}

pub fn update_animations(
    mut anim_q: Query<(
        &mut AnimationManager,
        &mut AnimationKey,
        &mut Handle<Image>,
        &mut TextureAtlas,
    )>,
) {
    for (mut manager, mut key, mut handle, mut atlas) in anim_q.iter_mut() {
        if manager.paused {
            continue;
        }
        let cur_node = manager.map.get(&key.0).unwrap();
        let cur_handle = cur_node.handle.clone();
        let cur_layout = cur_node.layout.clone();
        let cur_pace = cur_node.pace;
        let cur_length = cur_node.length;
        let cur_next = cur_node.next.clone();
        if handle.id() == cur_node.handle.id() {
            // We're on the correct animation, progress it
            manager.offset = (manager.offset + 1) % cur_pace.unwrap_or(2);
            if manager.offset == 0 {
                let old_idx = manager.idx;
                manager.idx = (manager.idx + 1) % cur_length;
                atlas.index = manager.idx as usize;
                if manager.idx < old_idx {
                    // We would be looping
                    if let Some(new_key) = cur_next {
                        if new_key != key.0 {
                            // We need to switch animations
                            manager.idx = 0;
                            manager.offset = 0;
                            let next_handle = manager.map.get(&new_key).unwrap().handle.clone();
                            *handle = next_handle;
                            *atlas = TextureAtlas {
                                layout: cur_layout,
                                index: 0,
                            };
                            *key = AnimationKey(new_key);
                        }
                    }
                }
            }
        } else {
            // We need to switch animations
            manager.idx = 0;
            manager.offset = 0;
            *handle = cur_handle;
            *atlas = TextureAtlas {
                layout: cur_layout,
                index: 0,
            };
        }
    }
}

pub struct MyAnimationPlugin;

impl Plugin for MyAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AnimationHead>();
        app.add_systems(
            FixedUpdate,
            (
                resolve_animation_head_stubs,
                update_animation_heads,
                materialize_animation_stubs,
            )
                .chain(),
        );
        app.add_systems(FixedUpdate, update_animations);
    }
}

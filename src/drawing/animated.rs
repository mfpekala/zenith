use bevy::{prelude::*, utils::HashMap};

#[derive(Debug)]
/// Information about a specific animation state that an object can be in
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

#[derive(Component, Debug)]
pub struct AnimationManager {
    pub map: HashMap<String, AnimatedNode>,
    pub idx: u8,
    pub offset: u8,
    pub flip_x: bool,
    pub flip_y: bool,
    pub paused: bool,
}
impl AnimationManager {
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

#[derive(Bundle)]
pub struct AnimationBundle {
    pub sprite_sheet: SpriteSheetBundle,
    pub manager: AnimationManager,
    pub val: AnimationKey,
}
impl AnimationBundle {
    pub fn new(initial_key: &str, manager: AnimationManager) -> Self {
        let initial_node = manager.map.get(initial_key).unwrap();
        Self {
            sprite_sheet: SpriteSheetBundle {
                texture: initial_node.handle.clone(),
                atlas: TextureAtlas {
                    layout: initial_node.layout.clone(),
                    index: 0,
                },
                ..default()
            },
            manager,
            val: AnimationKey(initial_key.to_string()),
        }
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
            manager.offset = (manager.offset + 1) % cur_pace.unwrap_or(4);
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
        app.add_systems(FixedUpdate, update_animations);
    }
}

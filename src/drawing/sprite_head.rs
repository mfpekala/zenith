use bevy::{prelude::*, render::view::RenderLayers};
use serde::{Deserialize, Serialize};

use crate::{
    editor::save::SaveMarker,
    uid::{UId, UIdMarker},
};

pub struct SpriteHeadStub {
    pub uid: UId,
    pub head: SpriteHead,
}

#[derive(Component, Default)]
pub struct SpriteHeadStubs(pub Vec<SpriteHeadStub>);

#[derive(Component, Clone, PartialEq, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct SpriteHead {
    pub path: String,
    pub render_layers: Vec<u8>,
    pub hidden: bool,
    pub offset: Vec3,
    pub scale: f32,
}
impl Default for SpriteHead {
    fn default() -> Self {
        Self {
            path: String::new(),
            render_layers: vec![],
            hidden: false,
            offset: Vec3::ZERO,
            scale: 1.0,
        }
    }
}

#[derive(Bundle, Default)]
pub struct SpriteHeadBundle {
    pub head: SpriteHead,
    spatial: SpatialBundle,
    save: SaveMarker,
}
impl SpriteHeadBundle {
    pub fn from_head(head: SpriteHead) -> Self {
        Self { head, ..default() }
    }
}

#[derive(Component)]
pub(super) struct SpriteBody {
    last_head: SpriteHead,
}

pub(super) fn resolve_sprite_head_stubs(
    mut commands: Commands,
    stubs: Query<(Entity, &SpriteHeadStubs)>,
) {
    for (eid, stubs) in stubs.iter() {
        for stub in stubs.0.iter() {
            commands.entity(eid).with_children(|parent| {
                parent.spawn((
                    UIdMarker(stub.uid),
                    SpriteHeadBundle::from_head(stub.head.clone()),
                ));
            });
        }
        commands.entity(eid).remove::<SpriteHeadStubs>();
    }
}

pub(super) fn update_sprite_heads(
    mut commands: Commands,
    heads: Query<(Entity, &SpriteHead, Option<&Children>)>,
    bodies: Query<(Entity, &mut SpriteBody)>,
    asset_server: Res<AssetServer>,
) {
    for (eid, head, children) in heads.iter() {
        match children {
            Some(children) => {
                for child in children {
                    let Ok((cid, body)) = bodies.get(*child) else {
                        commands.entity(eid).remove::<Children>();
                        continue;
                    };
                    if &body.last_head == head {
                        continue;
                    }
                    commands.entity(cid).insert((
                        SpriteBundle {
                            transform: Transform {
                                translation: head.offset,
                                scale: Vec3::new(head.scale, head.scale, 1.0),
                                ..default()
                            },
                            texture: asset_server.load(&head.path),
                            visibility: if head.hidden {
                                Visibility::Hidden
                            } else {
                                Visibility::Inherited
                            },
                            ..default()
                        },
                        RenderLayers::from_layers(&head.render_layers),
                        SpriteBody {
                            last_head: head.clone(),
                        },
                    ));
                }
            }
            None => {
                commands.entity(eid).with_children(|parent| {
                    parent.spawn((
                        SpriteBundle {
                            transform: Transform {
                                translation: head.offset,
                                scale: Vec3::new(head.scale, head.scale, 1.0),
                                ..default()
                            },
                            texture: asset_server.load(&head.path),
                            visibility: if head.hidden {
                                Visibility::Hidden
                            } else {
                                Visibility::Inherited
                            },
                            ..default()
                        },
                        RenderLayers::from_layers(&head.render_layers),
                        SpriteBody {
                            last_head: head.clone(),
                        },
                    ));
                });
            }
        }
    }
}

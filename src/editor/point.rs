use std::ops::Deref;

use bevy::{prelude::*, render::view::RenderLayers};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{
    drawing::layering::sprite_layer,
    input::MouseState,
    meta::game_state::{
        EditingMode, EditingState, EditorState, GameState, MetaState, SetGameState,
    },
    physics::dyno::IntMoveable,
};

use super::planet::EPlanet;

pub type EId = u64;

#[derive(Component, Clone, Debug, PartialEq, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct MyId(pub EId);
impl Deref for MyId {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Component, Clone, Debug, PartialEq, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub enum EPointKind {
    Rock,
    Field,
    #[default]
    Wild,
}
impl EPointKind {
    pub fn to_path(&self) -> String {
        match *self {
            Self::Field | Self::Rock => "sprites/editor/point.png".to_string(),
            Self::Wild => "sprites/editor/point_wild.png".to_string(),
        }
    }
}

#[derive(Component, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct EPoint {
    pub kind: EPointKind,
    pub size: u32,
    pub is_hovered: bool,
    pub is_selected: bool,
    pub drag_offset: Option<IVec2>,
}
impl EPoint {
    pub fn new(kind: EPointKind) -> Self {
        Self {
            kind,
            size: 3,
            is_hovered: false,
            is_selected: false,
            drag_offset: None,
        }
    }
}

#[derive(Component, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub(super) struct SelectSpriteMarker;

#[derive(Component, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct EPointSpriteMarker;

#[derive(Bundle)]
pub struct EPointBundle {
    pub id: MyId,
    pub point: EPoint,
    pub moveable: IntMoveable,
    pub sprite: SpriteBundle,
    pub psm: EPointSpriteMarker,
    pub render_layer: RenderLayers,
}
impl EPointBundle {
    fn new(pos: IVec2, kind: EPointKind, asset_server: &Res<AssetServer>) -> Self {
        let mut rng = thread_rng();
        Self {
            id: MyId(rng.gen::<EId>()),
            point: EPoint::new(kind.clone()),
            moveable: IntMoveable::new(pos.extend(2)),
            sprite: SpriteBundle {
                transform: Transform {
                    translation: pos.as_vec2().extend(0.1),
                    ..default()
                },
                texture: asset_server.load(kind.to_path()),
                ..default()
            },
            psm: EPointSpriteMarker,
            render_layer: sprite_layer(),
        }
    }

    pub fn spawn(
        commands: &mut ChildBuilder,
        asset_server: &Res<AssetServer>,
        pos: IVec2,
        kind: EPointKind,
    ) -> (Entity, EId) {
        let bund = Self::new(pos, kind, asset_server);
        let eid = bund.id.clone();
        (
            commands
                .spawn(bund)
                .with_children(|parent| {
                    parent.spawn((
                        SpriteBundle {
                            transform: Transform {
                                translation: Vec2::ZERO.extend(-0.1),
                                ..default()
                            },
                            texture: asset_server.load("sprites/editor/point_highlight.png"),
                            visibility: Visibility::Hidden,
                            ..default()
                        },
                        SelectSpriteMarker,
                        sprite_layer(),
                    ));
                })
                .id(),
            eid.0,
        )
    }
}

// Simply mark (in each point) whether it is hovered
pub(super) fn hover_points(
    mouse_state: Res<MouseState>,
    mut points: Query<(&GlobalTransform, &mut EPoint)>,
) {
    for (gt, mut point) in points.iter_mut() {
        let overlap_x = mouse_state
            .world_pos
            .x
            .abs_diff(gt.translation().x.round() as i32)
            < point.size;
        let overlap_y = mouse_state
            .world_pos
            .y
            .abs_diff(gt.translation().y.round() as i32)
            < point.size;
        point.is_hovered = overlap_x && overlap_y;
    }
}

/// What this "really" does is handle right click, which means variations of spawning points
pub(super) fn spawn_points(
    mut commands: Commands,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_state: Res<MouseState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gs: Res<GameState>,
    mut eplanets: Query<(&mut EPlanet, &IntMoveable)>,
    points: Query<(Entity, &EPoint, &IntMoveable, &MyId)>,
    mut gs_writer: EventWriter<SetGameState>,
    asset_server: Res<AssetServer>,
) {
    let Some(mode) = gs.get_editing_mode() else {
        return;
    };
    if !mouse_buttons.just_pressed(MouseButton::Right) {
        // See? it just handles right clicks
        return;
    }
    let get_entity = |eid: EId| {
        for point in points.iter() {
            if point.3 .0 == eid {
                return point.0;
            }
        }
        panic!("Bad eid");
    };
    match mode {
        EditingMode::Free => {
            // For now do nothing here
        }
        EditingMode::CreatingPlanet(planet_id) => {
            // Either closes the rock or places a new point
            let (mut eplanet, mv) = eplanets.get_mut(planet_id).unwrap();
            let closing = eplanet.rock_points.len() > 2
                && points
                    .get(get_entity(eplanet.rock_points[0]))
                    .unwrap()
                    .1
                    .is_hovered;
            if closing {
                gs_writer.send(SetGameState(GameState {
                    meta: MetaState::Editor(EditorState::Editing(EditingState {
                        mode: EditingMode::EditingPlanet(planet_id),
                    })),
                }));
            } else {
                commands.entity(planet_id).with_children(|mut parent| {
                    let (id, eid) = EPointBundle::spawn(
                        &mut parent,
                        &asset_server,
                        mouse_state.world_pos - mv.pos.truncate(),
                        EPointKind::Rock,
                    );
                    eplanet.rock_points.push(eid);
                });
            }
        }
        EditingMode::EditingPlanet(planet_id) => {
            let (mut eplanet, mv) = eplanets.get_mut(planet_id).unwrap();
            let spawning_at = mouse_state.world_pos - mv.pos.truncate();
            if keyboard.pressed(KeyCode::KeyF) {
                // ADDING A WILD POINT
                // These are points that later can be made into fields
                commands.entity(planet_id).with_children(|parent| {
                    let (id, eid) =
                        EPointBundle::spawn(parent, &asset_server, spawning_at, EPointKind::Wild);
                    eplanet.wild_points.push(id);
                });
            } else {
                // ADDING A ROCK POINT
                // Adds a new point in a pretty rational segment
                let mut closest_point = None;
                let mut closest_dist = i32::MAX;
                let mut closest_ix = 0;
                for (ix, id) in eplanet.rock_points.iter().enumerate() {
                    let (_, _, mv, _) = points.get(get_entity(*id)).unwrap();
                    let dist = mv.pos.truncate().distance_squared(spawning_at);
                    if closest_point.is_none() || dist < closest_dist {
                        closest_point = Some(*id);
                        closest_dist = dist;
                        closest_ix = ix as i32;
                    }
                }
                let anchor = points
                    .get(get_entity(closest_point.unwrap()))
                    .unwrap()
                    .2
                    .pos
                    .truncate();
                let anchor_vec = (spawning_at - anchor).as_vec2();
                let left_ix = (closest_ix - 1).rem_euclid(eplanet.rock_points.len() as i32);
                let right_ix = (closest_ix + 1).rem_euclid(eplanet.rock_points.len() as i32);
                let left_vec = points
                    .get(get_entity(eplanet.rock_points[left_ix as usize]))
                    .unwrap()
                    .2
                    .pos
                    .truncate()
                    - anchor;
                let left_vec = left_vec.as_vec2().normalize_or_zero();
                let right_vec = points
                    .get(get_entity(eplanet.rock_points[right_ix as usize]))
                    .unwrap()
                    .2
                    .pos
                    .truncate()
                    - anchor;
                let right_vec = right_vec.as_vec2().normalize_or_zero();
                let left_score = left_vec.dot(anchor_vec);
                let right_score = right_vec.dot(anchor_vec);
                println!("");
                let pos = if left_score < right_score {
                    right_ix as usize
                } else {
                    closest_ix as usize
                };

                commands.entity(planet_id).with_children(|mut parent| {
                    let (id, eid) = EPointBundle::spawn(
                        &mut parent,
                        &asset_server,
                        mouse_state.world_pos - mv.pos.truncate(),
                        EPointKind::Rock,
                    );
                    eplanet.rock_points.insert(pos, eid);
                });
            }
        }
    }
}

/// Really just handles left press/release, which usually means selecting/deselecting points
/// NOTE: Does NOT handle changing from editing/creating -> free, see planet_state_input for that
pub(super) fn select_points(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_state: Res<MouseState>,
    key_buttons: Res<ButtonInput<KeyCode>>,
    mut points: Query<(Entity, &mut EPoint, &IntMoveable, &GlobalTransform)>,
) {
    // If there's no press / release, do nothing
    if !mouse_buttons.just_pressed(MouseButton::Left)
        && !mouse_buttons.just_released(MouseButton::Left)
    {
        return;
    }
    // Figure out what points are already selected, and what points are hovered (if any)
    let mut selected = vec![];
    let mut hovered = vec![];
    for (id, point, _, _) in points.iter() {
        if point.is_selected {
            selected.push(id);
        }
        if point.is_hovered {
            hovered.push(id);
        }
    }
    // Helper functions
    let select_point =
        |id: Entity, q: &mut Query<(Entity, &mut EPoint, &IntMoveable, &GlobalTransform)>| {
            let (_, mut p, mv, gt) = q.get_mut(id).unwrap();
            p.is_selected = true;
            let gt2 = IVec2::new(gt.translation().x as i32, gt.translation().y as i32);
            let standard_off = gt2 - mouse_state.world_pos;
            let parent_tran = gt2 - mv.pos.truncate();
            p.drag_offset = Some(standard_off - parent_tran);
        };
    let deselect_point =
        |id: Entity, q: &mut Query<(Entity, &mut EPoint, &IntMoveable, &GlobalTransform)>| {
            let (_, mut p, _, _) = q.get_mut(id).unwrap();
            p.is_selected = false;
            p.drag_offset = None;
        };
    // Finally interpret the input
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if !key_buttons.pressed(KeyCode::ShiftLeft) {
            let deselecting = selected
                .clone()
                .into_iter()
                .filter(|p| !hovered.contains(p));
            for id in deselecting {
                deselect_point(id, &mut points);
            }
        } else {
            for id in selected.iter() {
                // Selecting the already selected points restarts their drag with the new offset
                select_point(*id, &mut points);
            }
        }
        for id in hovered {
            select_point(id, &mut points);
        }
    } else if !mouse_buttons.pressed(MouseButton::Left) {
        for id in selected {
            let (_, mut p, _, _) = points.get_mut(id).unwrap();
            p.drag_offset = None;
        }
    }
}

/// Handy keyboard shortcuts for point selection
pub(super) fn point_select_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_state: Res<MouseState>,
    mut points: Query<(Entity, &mut EPoint, &IntMoveable, &GlobalTransform, &MyId)>,
    eplanets: Query<&EPlanet>,
    gs: Res<GameState>,
) {
    let planet_id = match gs.get_editing_mode() {
        Some(EditingMode::CreatingPlanet(id)) => id,
        Some(EditingMode::EditingPlanet(id)) => id,
        _ => return,
    };
    let Ok(eplanet) = eplanets.get(planet_id) else {
        return;
    };
    // Helper functions
    let select_point =
        |eid: EId, q: &mut Query<(Entity, &mut EPoint, &IntMoveable, &GlobalTransform, &MyId)>| {
            let entity = q.iter().find(|p| p.4 .0 == eid).unwrap().0;
            let (_, mut p, mv, gt, eid) = q.get_mut(entity).unwrap();
            p.is_selected = true;
            let gt2 = IVec2::new(gt.translation().x as i32, gt.translation().y as i32);
            let standard_off = gt2 - mouse_state.world_pos;
            let parent_tran = gt2 - mv.pos.truncate();
            p.drag_offset = Some(standard_off - parent_tran);
        };
    let deselect_point =
        |id: Entity,
         q: &mut Query<(Entity, &mut EPoint, &IntMoveable, &GlobalTransform, &MyId)>| {
            let (_, mut p, _, _, eid) = q.get_mut(id).unwrap();
            p.is_selected = false;
            p.drag_offset = None;
        };

    if (keyboard.just_pressed(KeyCode::SuperLeft) && keyboard.pressed(KeyCode::KeyR))
        || (keyboard.just_pressed(KeyCode::KeyR) && keyboard.pressed(KeyCode::SuperLeft))
    {
        for id in eplanet.rock_points.iter() {
            select_point(*id, &mut points);
        }
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        let all_ids: Vec<Entity> = points.iter().map(|thing| thing.0).collect();
        for id in all_ids {
            deselect_point(id, &mut points);
        }
    }
}

/// Toggle the visibility of the select marker in response to the points selection status
pub(super) fn show_select_markers(
    points: Query<(&EPoint, &Children)>,
    mut select_markers: Query<&mut Visibility, With<SelectSpriteMarker>>,
) {
    for point in points.iter() {
        for child in point.1 {
            if let Ok(mut sm) = select_markers.get_mut(*child) {
                if point.0.is_selected {
                    *sm = Visibility::Inherited;
                } else {
                    *sm = Visibility::Hidden;
                }
            }
        }
    }
}

pub(super) fn delete_points(
    mut commands: Commands,
    mut eplanets: Query<(Entity, &mut EPlanet)>,
    points: Query<(Entity, &EPoint, &Parent, &MyId)>,
    key_buttons: Res<ButtonInput<KeyCode>>,
    gs: Res<GameState>,
    mut gs_writer: EventWriter<SetGameState>,
) {
    let get_entity = |eid: EId| {
        for point in points.iter() {
            if point.3 .0 == eid {
                return point.0;
            }
        }
        panic!("Bad eid");
    };
    if key_buttons.pressed(KeyCode::Backspace) {
        // Despawn the point, and then remove it from it's parent rock/field list
        for (id, p, parent_ref, eid) in points.iter() {
            if p.is_selected {
                match p.kind {
                    EPointKind::Rock => {
                        let mut parent = eplanets.get_mut(parent_ref.get()).unwrap().1;
                        parent.rock_points.retain(|x| *x != eid.0);
                    }
                    EPointKind::Field => {
                        let parent = points.get(parent_ref.get()).unwrap().2;
                        let mut eplanet = eplanets.get_mut(parent.get()).unwrap().1;
                        for field in eplanet.fields.iter_mut() {
                            field.field_points.retain(|x| *x != id);
                        }
                    }
                    EPointKind::Wild => {
                        let mut parent = eplanets.get_mut(parent_ref.get()).unwrap().1;
                        parent.wild_points.retain(|x| *x != id);
                    }
                }
                commands.entity(parent_ref.get()).remove_children(&[id]);
                commands.entity(id).despawn_recursive();
            }
        }
        // Check to see if any planets have fewer than three rock points. If so,
        // delete them
        let mut purged_planets = vec![];
        for (planet_id, eplanet) in eplanets.iter() {
            if eplanet.rock_points.len() < 3 {
                purged_planets.push(planet_id);
            }
            // Also check if any fields have less than three points. If so, delete them
        }
        for planet_id in purged_planets {
            // Also need to switch to free if we're deleting the creating or editing planet
            match gs.get_editing_mode() {
                Some(EditingMode::CreatingPlanet(id)) | Some(EditingMode::EditingPlanet(id)) => {
                    if planet_id == id {
                        gs_writer.send(SetGameState(EditingMode::Free.to_game_state()));
                    }
                }
                _ => (),
            }
            commands.entity(planet_id).despawn_recursive();
        }
    }
}

pub(super) fn move_points(
    mouse_state: Res<MouseState>,
    mut points: Query<(&EPoint, &mut IntMoveable)>,
) {
    for (p, mut mv) in points.iter_mut() {
        if let Some(offset) = p.drag_offset {
            mv.pos = (mouse_state.world_pos + offset).extend(mv.pos.z);
        }
    }
}

#[derive(Component, Debug, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub(super) struct ChangeEPointKind(pub EPointKind);

pub(super) fn update_point_sprites(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut points: Query<(Entity, &ChangeEPointKind, &mut Handle<Image>), With<EPoint>>,
) {
    for (id, change, mut hand) in points.iter_mut() {
        *hand = asset_server.load(change.0.to_path());
        commands.entity(id).remove::<ChangeEPointKind>();
    }
}

use std::{fs, ops::Deref};

use bevy::{ecs::system::SystemState, prelude::*, sprite::Mesh2dHandle, utils::HashSet};

use super::{
    help::HelpBarEvent,
    planet::EPlanet,
    point::{EPoint, SelectSpriteMarker},
    start_goal::{EGoal, EStart},
    EditingSceneRoot,
};
use crate::drawing::{
    bordered_mesh::{BorderMeshType, BorderedMesh},
    sprite_mat::SpriteMaterial,
};

#[derive(Event)]
pub(super) struct SaveEditorEvent;

#[derive(Event)]
pub(super) struct LoadEditorEvent;

fn unfuck_serialization(fucked: String) -> String {
    let mut result = String::new();
    let mut it = fucked.lines().into_iter();

    loop {
        let Some(line) = it.next() else {
            break;
        };
        let processing = line.contains("zenith::physics::dyno::IntMoveable");
        result.push_str(line);
        result.push('\n');

        if processing {
            let mut ihatemylife = vec![];
            for _ in 0..14 {
                ihatemylife.push(it.next().unwrap());
            }
            let x1 = ihatemylife[1].split(":").nth(1).unwrap().replace(",", "");
            let x1 = x1.trim();
            let x1 = x1.parse::<f32>().unwrap();
            let y1 = ihatemylife[2].split(":").nth(1).unwrap().replace(",", "");
            let y1 = y1.trim();
            let y1 = y1.parse::<f32>().unwrap();
            let x2 = ihatemylife[5].split(":").nth(1).unwrap().replace(",", "");
            let x2 = x2.trim();
            let x2 = x2.parse::<f32>().unwrap();
            let y2 = ihatemylife[6].split(":").nth(1).unwrap().replace(",", "");
            let y2 = y2.trim();
            let y2 = y2.parse::<f32>().unwrap();
            let z2 = ihatemylife[7].split(":").nth(1).unwrap().replace(",", "");
            let z2 = z2.trim();
            let z2 = z2.parse::<f32>().unwrap();
            let x3 = ihatemylife[10].split(":").nth(1).unwrap().replace(",", "");
            let x3 = x3.trim();
            let x3 = x3.parse::<f32>().unwrap();
            let y3 = ihatemylife[11].split(":").nth(1).unwrap().replace(",", "");
            let y3 = y3.trim();
            let y3 = y3.parse::<f32>().unwrap();
            result.push_str(&format!(
                r#"
    vel: ({}, {}),
    pos: ({}, {}, {}),
    rem: ({}, {}),
),
            "#,
                x1, y1, x2, y2, z2, x3, y3
            ))
        }

        let processing = line.contains("points: [")
            && !line.contains("rock")
            && !line.contains("wild")
            && !line.contains("field");
        while processing {
            let first = it.next().unwrap();
            if first.contains("]") {
                result.push_str(first);
                result.push('\n');
                break;
            }
            let x = it.next().unwrap().split(":").nth(1);
            let x = x.unwrap().replace(",", "");
            let x = x.trim();
            let x = x.parse::<f32>().unwrap();
            let y = it
                .next()
                .unwrap()
                .split(":")
                .nth(1)
                .unwrap()
                .replace(",", "");
            let y = y.trim();
            let y = y.parse::<f32>().unwrap();
            result.push_str(&format!("({}, {}),", x, y));
            it.next();
        }

        let processing = line.contains("size: (") || line.contains("bounds: (");
        if processing {
            let x = it.next().unwrap().split(":").nth(1);
            let x = x.unwrap().replace(",", "");
            let x = x.trim();
            let x = x.parse::<i32>().unwrap();
            let y = it
                .next()
                .unwrap()
                .split(":")
                .nth(1)
                .unwrap()
                .replace(",", "");
            let y = y.trim();
            let y = y.parse::<i32>().unwrap();
            result.push_str(&format!("{}, {},", x, y));
        }

        let processing =
            line.contains("vel: (") || line.contains("scroll: (") || line.contains("dir: (");
        if processing {
            let x = it.next().unwrap().split(":").nth(1);
            let x = x.unwrap().replace(",", "");
            let x = x.trim();
            let x = x.parse::<f32>().unwrap();
            let y = it
                .next()
                .unwrap()
                .split(":")
                .nth(1)
                .unwrap()
                .replace(",", "");
            let y = y.trim();
            let y = y.parse::<f32>().unwrap();
            result.push_str(&format!("{}, {},", x, y));
        }
    }

    result
}

pub(super) fn save_editor(
    world: &mut World,
    params: &mut SystemState<(
        EventReader<SaveEditorEvent>,
        Query<Entity, With<EditingSceneRoot>>,
        Query<&Children>,
    )>,
) {
    let (mut events, eroot, children) = params.get(world);
    if events.read().count() <= 0 {
        return;
    }
    let Ok(eroot) = eroot.get_single() else {
        return;
    };
    let mut keep = HashSet::new();
    keep.insert(eroot);
    for id in children.iter_descendants(eroot) {
        keep.insert(id);
    }
    let mut scene = DynamicSceneBuilder::from_world(&world)
        .deny::<Handle<SpriteMaterial>>()
        .deny::<Handle<Mesh>>()
        .deny::<Handle<Image>>()
        .deny::<Mesh2dHandle>()
        .deny_all_resources()
        .extract_entities(world.iter_entities().map(|entity| entity.id()))
        .build();
    scene
        .entities
        .retain(|entity| keep.contains(&entity.entity));
    scene.resources.clear();

    let type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = type_registry.deref();
    let serialized_scene = scene.serialize_ron(type_registry);

    match serialized_scene {
        Ok(text) => {
            fs::write("assets/test.scn.ron", text.clone()).expect("Unable to write file");
            let unfucked = unfuck_serialization(text);
            world.send_event(HelpBarEvent("Saved scene successfully".to_string()));
            fs::write("assets/test.scn.ron", unfucked.clone()).expect("Unable to write file");
        }
        Err(e) => {
            println!("{:?}", e);
            world.send_event(HelpBarEvent("Failed to save scene".to_string()));
        }
    }
}

pub(super) fn load_editor(
    mut loads: EventReader<LoadEditorEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    root: Query<Entity, With<EditingSceneRoot>>,
) {
    if loads.read().count() <= 0 {
        return;
    }
    let root = root.get_single().unwrap();
    commands.entity(root).despawn_recursive();
    commands.spawn((
        DynamicSceneBundle {
            scene: asset_server.load("test.scn.ron"),
            ..default()
        },
        Name::new("NewRoot"),
    ));
}

pub(super) fn fix_after_load(
    mut commands: Commands,
    dynamic: Query<(Entity, &Handle<DynamicScene>, Option<&Children>)>,
    names: Query<(Entity, &Name)>,
) {
    for dyno in dynamic.iter() {
        if dyno.2.is_some() {
            commands
                .entity(dyno.0)
                .remove_children(&dyno.2.clone().unwrap());
            for child in dyno.2.unwrap().iter() {
                commands.entity(*child).remove_parent();
            }
        }
    }
    for name in names.iter() {
        if name.1.as_str() == "EditingRoot" {
            commands.entity(name.0).insert(EditingSceneRoot);
        }
    }
}

#[derive(Event)]
pub struct CleanupLoadEvent;

pub(super) fn cleanup_load(
    mut commands: Commands,
    dynamic: Query<(Entity, &Handle<DynamicScene>)>,
    mut reader: EventReader<CleanupLoadEvent>,
) {
    if reader.read().count() > 0 {
        for thing in dynamic.iter() {
            commands.entity(thing.0).despawn_recursive();
        }
    }
}

pub(super) fn connect_parents(
    mut commands: Commands,
    eroot: Query<Entity, With<EditingSceneRoot>>,
    orphan_planets: Query<Entity, (With<EPlanet>, Without<Parent>)>,
    orphan_start: Query<Entity, (With<EStart>, Without<Parent>)>,
    orphan_goal: Query<Entity, (With<EGoal>, Without<Parent>)>,
) {
    let Ok(eroot) = eroot.get_single() else {
        return;
    };
    for id in orphan_planets
        .iter()
        .chain(orphan_start.iter())
        .chain(orphan_goal.iter())
    {
        commands.entity(eroot).push_children(&[id]);
    }
}

pub(super) fn resolve_holes(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    epoint_sprite: Query<(Entity, &EPoint), Without<Handle<Image>>>,
    epoint_sprite_select: Query<(Entity, &SelectSpriteMarker), Without<Handle<Image>>>,
    bms: Query<&BorderedMesh>,
    border_meshes: Query<
        (Entity, &BorderMeshType, &Parent),
        Or<(Without<Handle<SpriteMaterial>>, Without<Mesh2dHandle>)>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<SpriteMaterial>>,
) {
    for (id, epoint) in epoint_sprite.iter() {
        commands
            .entity(id)
            .insert(asset_server.load::<Image>(epoint.kind.to_path()));
    }
    for (id, _) in epoint_sprite_select.iter() {
        commands
            .entity(id)
            .insert(asset_server.load::<Image>("sprites/editor/point_highlight.png"));
    }
    for (id, kind, parent) in border_meshes.iter() {
        let bm = bms.get(parent.get()).unwrap();
        let is_inner = &kind.0 == "inner";
        let Some((new_mesh_handle, new_sprite_handle)) =
            bm.regen(is_inner, &asset_server, &mut meshes, &mut mats)
        else {
            continue;
        };
        commands.entity(id).insert(new_mesh_handle);
        commands.entity(id).insert(new_sprite_handle);
    }
}

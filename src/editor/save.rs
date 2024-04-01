use std::{fs, ops::Deref};

use bevy::{ecs::system::SystemState, prelude::*, sprite::Mesh2dHandle, utils::HashSet};

use crate::drawing::{bordered_mesh::BorderedMesh, sprite_mat::SpriteMaterial};

use super::{
    help::HelpBarEvent,
    planet::EPlanet,
    start_goal::{EGoal, EStart},
    EditingSceneRoot,
};

#[derive(Event)]
pub(super) struct SaveEditorEvent;

#[derive(Event)]
pub(super) struct LoadEditorEvent;

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
    fs::write(
        "assets/test.scn.ron",
        serialized_scene.clone().unwrap_or("None".to_string()),
    )
    .expect("Unable to write file");

    match serialized_scene {
        Ok(_) => {
            world.send_event(HelpBarEvent("Saved scene successfully".to_string()));
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
) {
    if loads.read().count() <= 0 {
        return;
    }
    commands.spawn(DynamicSceneBundle {
        // Scenes are loaded just like any other asset.
        scene: asset_server.load("test.scn.ron"),
        ..default()
    });
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

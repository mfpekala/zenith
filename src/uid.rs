use bevy::{prelude::*, utils::hashbrown::HashMap};
use rand::Rng;
use serde::{Deserialize, Serialize};

pub type UId = u64;

#[derive(Component, Clone, Debug, PartialEq, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct UIdMarker(pub UId);
impl std::ops::Deref for UIdMarker {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Resource, Default)]
pub struct UIdTranslator {
    map: HashMap<UId, Entity>,
    cleanup_countdown: u32,
}
impl UIdTranslator {
    const CLEANUP_INTERVAL: u32 = 5000;

    pub fn get_entity(&self, uid: UId) -> Option<Entity> {
        match self.map.get(&uid) {
            Some(ent) => Some(*ent),
            None => None,
        }
    }
}

fn update_translator(mut translator: ResMut<UIdTranslator>, things: Query<(Entity, &UIdMarker)>) {
    // TODO: See if this is too slow, and if so, play around with Added filter maybe?
    if translator.cleanup_countdown == 0 {
        let mut new_map = HashMap::new();
        for (eid, uid) in things.iter() {
            new_map.insert(uid.0, eid);
        }
        translator.map = new_map;
        translator.cleanup_countdown = UIdTranslator::CLEANUP_INTERVAL;
    } else {
        for (eid, uid) in things.iter() {
            translator.map.insert(uid.0, eid);
        }
    }
}

pub struct UIdPlugin;

impl Plugin for UIdPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UIdTranslator::default());
        app.add_systems(PreUpdate, update_translator);
    }
}

pub fn fresh_uid() -> UId {
    rand::thread_rng().gen::<crate::uid::UId>()
}

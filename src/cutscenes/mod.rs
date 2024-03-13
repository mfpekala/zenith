use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use self::chapter_one::register_chapter_one;

mod chapter_one;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum ChapterOneCutscenes {
    Alarm,
    WalkToWork,
}

#[derive(Resource, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum Cutscene {
    None,
    One(ChapterOneCutscenes),
}
pub struct CutsceneCase(pub Cutscene);

#[derive(Event)]
pub struct StartCutscene(pub Cutscene);

fn translate_cutscenes(mut start_reader: EventReader<StartCutscene>, mut res: ResMut<Cutscene>) {
    if let Some(cutscene) = start_reader.read().last() {
        *res = cutscene.0;
    }
}

pub fn is_in_any_cutscene(res: Res<Cutscene>) -> bool {
    match *res {
        Cutscene::None => false,
        _ => true,
    }
}

pub fn is_not_in_cutscene(res: Res<Cutscene>) -> bool {
    match *res {
        Cutscene::None => true,
        _ => false,
    }
}

pub struct CutscenesPlugin;

impl Plugin for CutscenesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Cutscene::None);
        app.add_event::<StartCutscene>();

        app.add_systems(FixedUpdate, translate_cutscenes);

        register_chapter_one(app);
    }
}

#[macro_export]
macro_rules! when_cutscene_started {
    ($cutscene: expr, $fname: ident) => {
        pub(super) fn $fname(mut reader: EventReader<crate::cutscenes::StartCutscene>) -> bool {
            if let Some(cutscene) = reader.read().last() {
                cutscene.0 == $cutscene
            } else {
                false
            }
        }
    };
}

#[macro_export]
macro_rules! is_in_cutscene {
    ($cutscene: expr, $fname: ident) => {
        pub(super) fn $fname(res: Res<crate::cutscenes::Cutscene>) -> bool {
            *res == $cutscene
        }
    };
}

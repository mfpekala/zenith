use self::chapter_one::register_chapter_one;
use crate::meta::{
    consts::TuneableConsts,
    game_state::{GameState, LevelState, MetaState, SetGameState},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

mod chapter_one;

#[derive(Component)]
/// Marks components that should be removed when a cutscene is over
pub struct CutsceneMarker;

pub fn clear_cutscene_entities(commands: &mut Commands, bgs: &Query<Entity, With<CutsceneMarker>>) {
    for id in bgs.iter() {
        commands.entity(id).despawn_recursive();
    }
}

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

#[derive(Event)]
pub struct StopCutscene;

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

#[derive(Component)]
struct PlayDelay(pub Timer);
fn play_setup(mut commands: Commands) {
    commands.spawn(PlayDelay(Timer::from_seconds(0.2, TimerMode::Once)));
}
fn play_update(
    mut commands: Commands,
    mut play_delay: Query<(Entity, &mut PlayDelay)>,
    time: Res<Time>,
    mut gs_writer: EventWriter<SetGameState>,
    mut cutscene_starter: EventWriter<StartCutscene>,
    mut cutscene_stopper: EventWriter<StopCutscene>,
    cutscene_res: Res<Cutscene>,
    tune: Res<TuneableConsts>,
) {
    let playing = Cutscene::One(ChapterOneCutscenes::Alarm);
    let Ok((id, mut pd)) = play_delay.get_single_mut() else {
        // This code lets us restart the cutscene whenever one of the consts changes
        if tune.is_changed() {
            cutscene_stopper.send(StopCutscene);
        }
        if *cutscene_res == Cutscene::None {
            cutscene_starter.send(StartCutscene(playing));
        }
        return;
    };
    // This weird timer based code lets us simulate slipping into the cutscene
    pd.0.tick(time.delta());
    if pd.0.finished() {
        gs_writer.send(SetGameState(GameState {
            meta: MetaState::Level(LevelState::fresh_from_id("L1".to_string())),
        }));
        cutscene_starter.send(StartCutscene(playing));
        commands.entity(id).despawn();
    }
}

pub struct CutscenesPlugin;

impl Plugin for CutscenesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Cutscene::None);
        app.add_event::<StartCutscene>();
        app.add_event::<StopCutscene>();

        app.add_systems(Startup, play_setup);
        app.add_systems(Update, play_update);
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

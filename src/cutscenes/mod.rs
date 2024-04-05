use self::chapter_one::{register_chapter_one, walk_to_work::Walk2WorkHot};
use crate::{
    drawing::effects::TriggerFadeToBlack,
    meta::{
        consts::TuneableConsts,
        game_state::{GameState, LevelState, MetaState, SetGameState},
    },
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

mod chapter_one;

#[derive(Component)]
/// Marks components that should be removed when a cutscene is over
pub struct CutsceneMarker;

#[derive(Component)]
/// Marks components that should be removed when the current cutscene is not in the provided list
pub struct DurableCutsceneMarker(pub Vec<Cutscene>);

pub fn clear_cutscene_entities(
    commands: &mut Commands,
    next_cutscene: Cutscene,
    css: &Query<Entity, With<CutsceneMarker>>,
    dcss: &Query<(Entity, &DurableCutsceneMarker)>,
) {
    for id in css.iter() {
        commands.entity(id).despawn_recursive();
    }
    for (id, durable) in dcss {
        if !durable.0.contains(&next_cutscene) {
            commands.entity(id).despawn_recursive();
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum ChapterOneCutscenes {
    Alarm,
    WalkToWork,
}

#[derive(Resource, Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Cutscene {
    None,
    One(ChapterOneCutscenes),
}
pub struct CutsceneCase(pub Cutscene);

#[derive(Event)]
pub struct StartCutscene(pub Cutscene);

#[derive(Event)]
/// Stops a cutscene, containing the next cutscene to start
pub struct StopCutscene(pub Cutscene);

#[derive(Component)]
/// Fades to black, then stops the current cutscene and starts the next cutscene, then unfades
pub struct CutsceneFadeKiller {
    pub kill_to: Cutscene,
    pub timer: Timer,
    pub fade_started_in: bool,
    pub padding_timer: Timer,
    pub fade_started_out: bool,
}
impl CutsceneFadeKiller {
    pub fn duration() -> f32 {
        0.75
    }

    pub fn new(cutscene: Cutscene) -> Self {
        Self {
            kill_to: cutscene,
            timer: Timer::from_seconds(Self::duration(), TimerMode::Once),
            fade_started_in: false,
            padding_timer: Timer::from_seconds(Self::duration() + 0.25, TimerMode::Once),
            fade_started_out: false,
        }
    }
}

fn update_fade_killers(
    mut commands: Commands,
    mut ftb_writer: EventWriter<TriggerFadeToBlack>,
    mut stop_writer: EventWriter<StopCutscene>,
    mut killers: Query<(Entity, &mut CutsceneFadeKiller)>,
    time: Res<Time>,
) {
    for (id, mut killer) in killers.iter_mut() {
        if !killer.fade_started_in {
            ftb_writer.send(TriggerFadeToBlack((1.0, CutsceneFadeKiller::duration())));
            killer.fade_started_in = true;
        }
        killer.timer.tick(time.delta());
        if killer.timer.finished() && !killer.fade_started_out {
            ftb_writer.send(TriggerFadeToBlack((0.0, CutsceneFadeKiller::duration())));
            stop_writer.send(StopCutscene(killer.kill_to));
            killer.fade_started_out = true;
        }
        killer.padding_timer.tick(time.delta());
        if killer.padding_timer.finished() {
            commands.entity(id).despawn_recursive();
        }
    }
}

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

#[allow(unused)]
#[derive(Component)]
struct PlayDelay(pub Timer);
#[allow(unused)]
fn play_setup(mut commands: Commands) {
    commands.spawn(PlayDelay(Timer::from_seconds(0.2, TimerMode::Once)));
}
#[allow(unused)]
fn play_update(
    mut commands: Commands,
    mut play_delay: Query<(Entity, &mut PlayDelay)>,
    time: Res<Time>,
    mut gs_writer: EventWriter<SetGameState>,
    mut cutscene_starter: EventWriter<StartCutscene>,
    mut cutscene_stopper: EventWriter<StopCutscene>,
    cutscene_res: Res<Cutscene>,
    tune: Res<TuneableConsts>,
    walk2work: Res<Walk2WorkHot>,
) {
    let playing = Cutscene::One(ChapterOneCutscenes::WalkToWork);
    let Ok((id, mut pd)) = play_delay.get_single_mut() else {
        // This code lets us restart the cutscene whenever one of the consts changes
        if tune.is_changed() || walk2work.is_changed() {
            cutscene_stopper.send(StopCutscene(Cutscene::None));
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

        // app.add_systems(Startup, _play_setup);
        // app.add_systems(Update, _play_update);
        app.add_systems(FixedUpdate, update_fade_killers);
        app.add_systems(FixedUpdate, translate_cutscenes.after(update_fade_killers));

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

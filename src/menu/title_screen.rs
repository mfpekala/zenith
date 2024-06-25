use crate::{
    drawing::{
        animation::{AnimationManager, SpriteInfo},
        effects::ScreenEffectManager,
        layering::menu_layer_u8,
        text::{Flashing, TextAlign, TextBoxBundle, TextWeight},
    },
    environment::background::{BgEffect, BgManager},
    meta::game_state::{EditingState, EditorState, GameState, MenuState, MetaState, SetMetaState},
    when_becomes_false, when_becomes_true,
};
use bevy::prelude::*;

use super::{
    placement::{GameRelativePlacement, GameRelativePlacementBundle},
    update_any_menu,
};

/// Root of the title screen, will be destroyed on destroy
#[derive(Component)]
struct TitleScreenRoot;

#[derive(Component)]
struct TitleScreenDeath;

fn setup_title_screen(mut commands: Commands) {
    commands
        .spawn((
            SpatialBundle::default(),
            Name::new("title_screen_root"),
            TitleScreenRoot,
        ))
        .with_children(|parent| {
            // ZENITH
            let mut zenith_man = AnimationManager::single_static(SpriteInfo {
                path: "sprites/menu/title/ZENITH.png".to_string(),
                size: UVec2::new(185, 53),
                ..default()
            });
            zenith_man.set_render_layers(vec![menu_layer_u8()]);
            parent.spawn((
                GameRelativePlacementBundle::new(IVec3::new(0, 40, 0), 0.75),
                zenith_man,
            ));
            // Press any button to start
            let text_bund = (
                TextBoxBundle::new_menu_text(
                    "* press enter to start *",
                    36.0,
                    GameRelativePlacement::new(IVec3::new(0, -50, 0), 0.5),
                    Color::WHITE,
                    TextWeight::default(),
                    TextAlign::Center,
                ),
                Flashing::new(1.0, 0.5),
            );
            parent.spawn(text_bund);
        });
}

fn update_title_screen(
    mut commands: Commands,
    death: Query<Entity, With<TitleScreenDeath>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut gs_writer: EventWriter<SetMetaState>,
    root: Query<Entity, With<TitleScreenRoot>>,
    mut bg_manager: ResMut<BgManager>,
    mut screen_effects: ResMut<ScreenEffectManager>,
) {
    let Ok(root) = root.get_single() else {
        return;
    };
    if keys.just_pressed(KeyCode::Enter) && death.iter().len() == 0 {
        if keys.pressed(KeyCode::KeyE) {
            // Activate the editor by pressing E
            gs_writer.send(SetMetaState(MetaState::Editor(EditorState::Editing(
                EditingState::blank(),
            ))));
            return;
        }
        bg_manager.queue_effect(BgEffect::default_menu_scroll(
            true,
            true,
            Some(MetaState::Menu(MenuState::ConstellationSelect)),
        ));
        bg_manager.queue_effect(BgEffect::default_menu_scroll(true, false, None));
        commands.entity(root).with_children(|parent| {
            parent.spawn(TitleScreenDeath);
        });
    }
}

fn destroy_title_screen(mut commands: Commands, markers: Query<Entity, With<TitleScreenRoot>>) {
    for eid in markers.iter() {
        commands.entity(eid).despawn_recursive();
    }
}

fn is_in_title_screen_helper(gs: &GameState) -> bool {
    match gs.meta {
        MetaState::Menu(menu_state) => match menu_state {
            MenuState::Title => true,
            _ => false,
        },
        _ => false,
    }
}
fn is_in_title_screen(gs: Res<GameState>) -> bool {
    is_in_title_screen_helper(&gs)
}
when_becomes_true!(is_in_title_screen_helper, entered_title_screen);
when_becomes_false!(is_in_title_screen_helper, left_title_screen);

pub fn register_title_screen(app: &mut App) {
    app.add_systems(Update, setup_title_screen.run_if(entered_title_screen));
    app.add_systems(Update, destroy_title_screen.run_if(left_title_screen));
    app.add_systems(
        Update,
        update_title_screen
            .run_if(is_in_title_screen)
            .after(update_any_menu),
    );
}

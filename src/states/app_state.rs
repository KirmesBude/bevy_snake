use bevy::prelude::*;

use crate::{components::PauseText, resources::Materials};

use super::game_state::GameState;

pub const SNAKE_APP_STATE_STARTUP: &'static str = "snake_app_state_startup";
pub const SNAKE_APP_STATE: &'static str = "snake_app_state";
#[derive(Clone, PartialEq, Eq)]
pub enum AppState {
    Setup,
    Menu,
    Game,
}

pub fn app_setup(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut app_state: ResMut<State<AppState>>,
) {
    commands.spawn(CameraUiBundle::default());
    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(Materials {
        head_material: materials.add(Color::rgb(0.7, 0.7, 0.7).into()),
        food_material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
        body_material: materials.add(Color::rgb(0.6, 0.6, 0.6).into()),
    });

    commands
        .spawn(TextBundle {
            visible: Visible {
                is_visible: false,
                ..Default::default()
            },
            style: Style {
                align_self: AlignSelf::Center, /* Center center ??? */
                ..Default::default()
            },
            text: Text {
                value: "Paused".to_string(),
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                style: TextStyle {
                    font_size: 200.0, /* TODO: does not give a shit about window scale */
                    color: Color::WHITE,
                    alignment: TextAlignment {
                        vertical: VerticalAlign::Center,
                        horizontal: HorizontalAlign::Center,
                    },
                },
            },
            ..Default::default()
        })
        .with(PauseText);

    app_state.set_next(AppState::Menu).unwrap();
}

pub fn menu_enter() {}

pub fn menu_update(mut state: ResMut<State<AppState>>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.pressed(KeyCode::Space) {
        state.set_next(AppState::Game).unwrap();
    }
}

pub fn menu_exit() {}

pub fn game_enter(mut state: ResMut<State<GameState>>) {
    state.set_next(GameState::Running).unwrap();
}

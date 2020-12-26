use bevy::prelude::*;

use crate::resources::{Fonts, Materials};

use super::game_state::GameState;

pub const SNAKE_APP_STATE_STARTUP: &str = "snake_app_state_startup";
pub const SNAKE_APP_STATE: &str = "snake_app_state";
#[derive(Clone, PartialEq, Eq)]
pub enum AppState {
    Setup,
    Menu,
    Game,
}

/* Setup */
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
    commands.insert_resource(Fonts {
        pause_font: asset_server.load("fonts/FiraSans-Bold.ttf").into(),
    });

    app_state.set_next(AppState::Menu).unwrap();
}

/* Menu */
pub fn menu_enter() {}

pub fn menu_update(mut state: ResMut<State<AppState>>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.pressed(KeyCode::Space) {
        state.set_next(AppState::Game).unwrap();
    }
}

pub fn menu_exit() {}

/* Game */
pub fn game_enter(mut state: ResMut<State<GameState>>) {
    state.set_next(GameState::Running).unwrap();
}

/* game_enter handled in game_state */

pub fn game_exit(mut state: ResMut<State<GameState>>) {
    state.set_next(GameState::None).unwrap();
}

use bevy::{
    ecs::{Commands, Entity, Query, Res, ResMut, State, With},
    input::Input,
    math::{Rect, Vec2, Vec3},
    prelude::{
        AssetServer, Assets, BuildChildren, ButtonBundle, Camera2dBundle, CameraUiBundle, Color,
        DespawnRecursiveExt, KeyCode, Text, TextBundle, Transform,
    },
    sprite::{ColorMaterial, Sprite},
    text::TextStyle,
    ui::{AlignItems, JustifyContent, Style, Val},
    window::Windows,
};

use crate::{
    components::{MenuButton, Position, Size},
    resources::{ButtonMaterials, Fonts, Materials},
};

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
        pause_font: asset_server.load("fonts/FiraSans-Bold.ttf"),
    });
    commands.insert_resource(ButtonMaterials {
        normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
        hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
        pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
    });

    app_state.set_next(AppState::Menu).unwrap();
}

/* Menu */
pub fn menu_enter(
    commands: &mut Commands,
    fonts: Res<Fonts>,
    button_materials: Res<ButtonMaterials>,
) {
    commands
        .spawn(ButtonBundle {
            style: Style {
                size: bevy::math::Size::new(Val::Px(150.0), Val::Px(65.0)),
                // center button
                margin: Rect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: button_materials.normal.clone(),
            ..Default::default()
        })
        .with(MenuButton)
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    value: "Press Space".to_string(),
                    font: fonts.pause_font.clone(),
                    style: TextStyle {
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        ..Default::default()
                    },
                },
                ..Default::default()
            });
        });
}

pub fn menu_update(mut state: ResMut<State<AppState>>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.pressed(KeyCode::Space) {
        state.set_next(AppState::Game).unwrap();
    }
}

pub fn menu_exit(commands: &mut Commands, menu_buttons: Query<Entity, With<MenuButton>>) {
    for entity in menu_buttons.iter() {
        commands.despawn_recursive(entity);
    }
}

/* Game */
pub fn game_enter(mut state: ResMut<State<GameState>>) {
    state.set_next(GameState::Running).unwrap();
}

pub const ARENA_WIDTH: u32 = 30;
pub const ARENA_HEIGHT: u32 = 30;

pub fn position_translation(windows: Res<Windows>, mut query: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.0,
        );
    }
}

pub fn sprite_scaling(windows: Res<Windows>, mut query: Query<(&Size, &mut Sprite)>) {
    let window = windows.get_primary().unwrap();
    for (size, mut sprite) in query.iter_mut() {
        sprite.size = Vec2::new(
            size.width * (window.width() as f32 / ARENA_WIDTH as f32),
            size.height * (window.height() as f32 / ARENA_HEIGHT as f32),
        );
    }
}

pub fn game_exit(mut state: ResMut<State<GameState>>) {
    state.set_next(GameState::None).unwrap();
}

use bevy::prelude::*;
use bevy::{core::FixedTimestep, render::pass::ClearColor};
use resources::{GameOverEvent, GrowthEvent};
use states::{
    app_state::{
        app_setup, game_enter, game_exit, menu_enter, menu_exit, menu_update, AppState,
        SNAKE_APP_STATE, SNAKE_APP_STATE_STARTUP,
    },
    game_state::{
        eat_food, enter_pause, exit_pause, game_over, position_translation, snake_direction,
        snake_eat_snake, snake_growth, snake_movement, spawn_food, spawn_snake, sprite_scaling,
        toggle_pause, wrapping_edges, GameState, SNAKE_GAME_STATE,
    },
};

mod components;
mod resources;
mod states;

#[bevy_main]
fn main() {
    App::build()
        /* Window initialization */
        .add_resource(WindowDescriptor {
            title: "Snake!".to_string(),
            width: 1200.0,
            height: 1200.0,
            ..Default::default()
        })
        /* Resources */
        .add_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_resource(State::new(AppState::Setup))
        .add_resource(State::new(GameState::None))
        /* Events */
        .add_event::<GrowthEvent>()
        .add_event::<GameOverEvent>()
        /* Startup */
        .add_stage_after(
            stage::STARTUP,
            SNAKE_APP_STATE_STARTUP,
            StateStage::<AppState>::default()
                .with_enter_stage(AppState::Setup, SystemStage::single(app_setup.system())),
        )
        /* Systems */
        .add_stage_after(
            stage::UPDATE,
            SNAKE_APP_STATE,
            StateStage::<AppState>::default()
                .with_enter_stage(AppState::Menu, SystemStage::single(menu_enter.system()))
                .with_update_stage(AppState::Menu, SystemStage::single(menu_update.system()))
                .with_exit_stage(AppState::Menu, SystemStage::single(menu_exit.system()))
                .with_enter_stage(
                    AppState::Game,
                    SystemStage::serial()
                        .with_system(game_enter.system())
                        .with_system(spawn_snake.system()),
                )
                .with_exit_stage(AppState::Game, SystemStage::single(game_exit.system())),
        )
        .add_stage_after(
            SNAKE_APP_STATE,
            SNAKE_GAME_STATE,
            StateStage::<GameState>::default()
                .with_enter_stage(
                    GameState::Pausing,
                    SystemStage::single(enter_pause.system()),
                )
                .with_update_stage(
                    GameState::Pausing,
                    SystemStage::single(toggle_pause.system()),
                )
                .with_exit_stage(GameState::Pausing, SystemStage::single(exit_pause.system()))
                .with_update_stage(
                    GameState::Running,
                    Schedule::default()
                        .with_stage(
                            "game_loop",
                            SystemStage::parallel()
                                .with_system(position_translation.system())
                                .with_system(sprite_scaling.system())
                                .with_system(snake_direction.system())
                                .with_system(eat_food.system())
                                .with_system(snake_growth.system())
                                .with_system(wrapping_edges.system())
                                .with_system(snake_eat_snake.system())
                                .with_system(game_over.system())
                                .with_system(toggle_pause.system()),
                        )
                        .with_stage(
                            "fixed_update_snake",
                            SystemStage::parallel()
                                .with_run_criteria(FixedTimestep::step(0.15))
                                .with_system(snake_movement.system()),
                        )
                        .with_stage(
                            "fixed_update_food",
                            SystemStage::parallel()
                                .with_run_criteria(FixedTimestep::step(1.0))
                                .with_system(spawn_food.system()),
                        ),
                ),
        )
        /* Plugins */
        .add_plugins(DefaultPlugins)
        .run();
}

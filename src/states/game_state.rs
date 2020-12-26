use crate::{
    components::{Direction, Food, PauseText, Position, Size, Snake, SnakeHead, SnakeSegment},
    resources::{GameOverEvent, GrowthEvent, Materials},
};
use bevy::{
    ecs::{Entity, Local, Query, ResMut, State, With},
    input::Input,
    math::Vec3,
    prelude::{
        BuildChildren, Children, Commands, DespawnRecursiveExt, EventReader, Events,
        GlobalTransform, KeyCode, Res, Sprite, SpriteBundle, Transform, Vec2, Visible,
    },
    window::Windows,
};

use super::app_state::AppState;

pub const SNAKE_GAME_STATE: &'static str = "snake_game_state";
#[derive(Clone, PartialEq, Eq)]
pub enum GameState {
    None,
    Running,
    Pausing,
}

pub fn spawn_snake(commands: &mut Commands, materials: Res<Materials>) {
    let snake_head = commands
        .spawn(SpriteBundle {
            material: materials.head_material.clone(),
            sprite: Sprite::new(Vec2::new(10.0, 10.0)),
            ..Default::default()
        })
        .with(SnakeHead)
        .with(SnakeSegment)
        .with(Position { x: 3, y: 3 })
        .with(Size::square(0.8))
        .current_entity()
        .unwrap();

    let snake_body = commands
        .spawn(SpriteBundle {
            material: materials.body_material.clone(),
            sprite: Sprite::new(Vec2::new(10.0, 10.0)),
            ..Default::default()
        })
        .with(SnakeSegment)
        .with(Position { x: 2, y: 3 })
        .with(Size::square(0.7))
        .current_entity()
        .unwrap();

    let snake_tail = commands
        .spawn(SpriteBundle {
            material: materials.body_material.clone(),
            sprite: Sprite::new(Vec2::new(10.0, 10.0)),
            ..Default::default()
        })
        .with(SnakeSegment)
        .with(Position { x: 1, y: 3 })
        .with(Size::square(0.7))
        .current_entity()
        .unwrap();

    let snake = commands
        .spawn((
            Snake {
                direction: Direction::Right,
                last_direction: Direction::Right,
            },
            Transform::default(),
            GlobalTransform::default(),
        ))
        .current_entity()
        .unwrap();

    commands.push_children(snake, &[snake_head, snake_body, snake_tail]);
}

const ARENA_WIDTH: u32 = 10;
const ARENA_HEIGHT: u32 = 10;

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

pub fn snake_direction(keyboard_input: Res<Input<KeyCode>>, mut snakes: Query<&mut Snake>) {
    for mut snake in snakes.iter_mut() {
        let direction = {
            if keyboard_input.pressed(KeyCode::Left) {
                Direction::Left
            } else if keyboard_input.pressed(KeyCode::Right) {
                Direction::Right
            } else if keyboard_input.pressed(KeyCode::Up) {
                Direction::Up
            } else if keyboard_input.pressed(KeyCode::Down) {
                Direction::Down
            } else {
                snake.direction
            }
        };

        if direction != snake.last_direction.opposite() {
            snake.direction = direction;
        }
    }
}

pub fn snake_movement(
    mut snakes: Query<(&mut Snake, &Children)>,
    mut positions: Query<&mut Position>,
) {
    for (mut snake, segments) in snakes.iter_mut() {
        /* Actual Movement */
        for two_segements in segments.windows(2).rev() {
            let prev = two_segements[1];
            let next = two_segements[0];

            let next_position = positions.get_mut(next).unwrap().clone();
            let mut prev_position = positions.get_mut(prev).unwrap();
            prev_position.x = next_position.x;
            prev_position.y = next_position.y;
        }

        let mut head_position = positions.get_mut(segments[0]).unwrap();
        match snake.direction {
            Direction::Left => head_position.x -= 1,
            Direction::Right => head_position.x += 1,
            Direction::Up => head_position.y += 1,
            Direction::Down => head_position.y -= 1,
        }

        snake.last_direction = snake.direction;
    }
}

pub fn spawn_food(commands: &mut Commands, materials: Res<Materials>, q: Query<&Position>) {
    let position = Position {
        x: (rand::random::<f32>() * ARENA_WIDTH as f32) as i32,
        y: (rand::random::<f32>() * ARENA_HEIGHT as f32) as i32,
    };

    //check position
    if q.iter().any(|p| p == &position) {
        return;
    }

    commands
        .spawn(SpriteBundle {
            material: materials.food_material.clone(),
            ..Default::default()
        })
        .with(Food)
        .with(position)
        .with(Size::square(0.8));
}

pub fn eat_food(
    commands: &mut Commands,
    mut growth_events: ResMut<Events<GrowthEvent>>,
    snakes: Query<(Entity, &Children), With<Snake>>,
    positions: Query<&Position>,
    food_positions: Query<(Entity, &Position), With<Food>>,
) {
    for (entity, segments) in snakes.iter() {
        let head = segments[0];
        let head_position = positions.get(head).unwrap();
        for (food_entity, food_position) in food_positions.iter() {
            if food_position == head_position {
                commands.despawn(food_entity);
                growth_events.send(GrowthEvent { snake: entity });
            }
        }
    }
}

pub fn snake_growth(
    commands: &mut Commands,
    materials: Res<Materials>,
    growth_events: Res<Events<GrowthEvent>>,
    mut growth_reader: Local<EventReader<GrowthEvent>>,
    positions: Query<&Position>,
    segments: Query<&Children>,
) {
    for growth_event in growth_reader.iter(&growth_events) {
        let snake_segments = segments.get(growth_event.snake).unwrap();

        let last_segment = snake_segments.last().unwrap();
        let last_segment_position = positions.get(*last_segment).unwrap();

        let snake_tail = commands
            .spawn(SpriteBundle {
                material: materials.body_material.clone(),
                sprite: Sprite::new(Vec2::new(10.0, 10.0)),
                ..Default::default()
            })
            .with(SnakeSegment)
            .with(last_segment_position.clone())
            .with(Size::square(0.7))
            .current_entity()
            .unwrap();

        commands.push_children(growth_event.snake, &[snake_tail]);
    }
}

pub fn wrapping_edges(mut positions: Query<&mut Position>) {
    for mut position in positions.iter_mut() {
        if position.x < 0 {
            position.x = (ARENA_WIDTH - 1) as i32;
        }
        if position.x > (ARENA_WIDTH - 1) as i32 {
            position.x = 0;
        }
        if position.y < 0 {
            position.y = (ARENA_HEIGHT - 1) as i32;
        }
        if position.y > (ARENA_HEIGHT - 1) as i32 {
            position.y = 0;
        }
    }
}

pub fn snake_eat_snake(
    mut game_over_event: ResMut<Events<GameOverEvent>>,
    positions: Query<&Position>,
    snakes: Query<(Entity, &Children), With<Snake>>,
    head_positions: Query<&Position, With<SnakeHead>>,
) {
    for (entity, segments) in snakes.iter() {
        for head_position in head_positions.iter() {
            for segment in &segments[1..] {
                if let Ok(segment_position) = positions.get(*segment) {
                    if head_position == segment_position {
                        game_over_event.send(GameOverEvent { snake: entity }); /* We send the entity that was "eaten" to be destroyed */
                    }
                }
            }
        }
    }
}

pub fn game_over(
    commands: &mut Commands,
    materials: Res<Materials>,
    game_over_events: Res<Events<GameOverEvent>>,
    mut game_over_reader: Local<EventReader<GameOverEvent>>,
    foods: Query<Entity, With<Food>>,
) {
    for game_over_event in game_over_reader.iter(&game_over_events) {
        commands.despawn_recursive(game_over_event.snake);

        for food in foods.iter() {
            commands.despawn(food);
        }

        spawn_snake(commands, materials);
        return;
    }
}

pub fn toggle_pause(
    app_state: Res<State<AppState>>,
    mut game_state: ResMut<State<GameState>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) && *app_state.current() == AppState::Game {
        match game_state.current() {
            GameState::Running => game_state.set_next(GameState::Pausing).unwrap(),
            GameState::Pausing => game_state.set_next(GameState::Running).unwrap(),
            _ => (),
        }
    }
}

pub fn enter_pause(mut pause_text: Query<&mut Visible, With<PauseText>>) {
    for mut visible in pause_text.iter_mut() {
        (*visible).is_visible = true;
    }
}

pub fn exit_pause(mut pause_text: Query<&mut Visible, With<PauseText>>) {
    for mut visible in pause_text.iter_mut() {
        (*visible).is_visible = false;
    }
}
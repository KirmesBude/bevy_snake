use bevy::prelude::*;
use bevy::{core::FixedTimestep, render::pass::ClearColor};

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
        /* Events */
        .add_event::<GrowthEvent>()
        .add_event::<GameOverEvent>()
        /* Startup */
        .add_startup_system(setup.system())
        .add_startup_stage(
            "game_setup",
            SystemStage::parallel().with_system(spawn_snake.system()),
        )
        /* Systems */
        .add_system(position_translation.system())
        .add_system(sprite_scaling.system())
        .add_system(snake_direction.system())
        .add_stage_after(
            stage::UPDATE,
            "fixed_update_snake",
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(0.15))
                .with_system(snake_movement.system()),
        )
        .add_stage_after(
            stage::UPDATE,
            "fixed_update_food",
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(1.0))
                .with_system(spawn_food.system()),
        )
        .add_system(eat_food.system())
        .add_system(snake_growth.system())
        .add_system(wrapping_edges.system())
        .add_system(snake_eat_snake.system())
        .add_system(game_over.system())
        /* Plugins */
        .add_plugins(DefaultPlugins)
        .run();
}

struct Materials {
    head_material: Handle<ColorMaterial>,
    food_material: Handle<ColorMaterial>,
    body_material: Handle<ColorMaterial>,
}

fn setup(commands: &mut Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(Materials {
        head_material: materials.add(Color::rgb(0.7, 0.7, 0.7).into()),
        food_material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
        body_material: materials.add(Color::rgb(0.6, 0.6, 0.6).into()),
    });
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}
struct Snake {
    direction: Direction,
    last_direction: Direction,
}

struct SnakeHead;

struct SnakeSegment;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct Position {
    x: i32,
    y: i32,
}

struct Size {
    width: f32,
    height: f32,
}

impl Size {
    fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

fn spawn_snake(commands: &mut Commands, materials: Res<Materials>) {
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

fn position_translation(windows: Res<Windows>, mut query: Query<(&Position, &mut Transform)>) {
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

fn sprite_scaling(windows: Res<Windows>, mut query: Query<(&Size, &mut Sprite)>) {
    let window = windows.get_primary().unwrap();
    for (size, mut sprite) in query.iter_mut() {
        sprite.size = Vec2::new(
            size.width * (window.width() as f32 / ARENA_WIDTH as f32),
            size.height * (window.height() as f32 / ARENA_HEIGHT as f32),
        );
    }
}

fn snake_direction(keyboard_input: Res<Input<KeyCode>>, mut snakes: Query<&mut Snake>) {
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

fn snake_movement(mut snakes: Query<(&mut Snake, &Children)>, mut positions: Query<&mut Position>) {
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

struct Food;

fn spawn_food(commands: &mut Commands, materials: Res<Materials>, q: Query<&Position>) {
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

struct GrowthEvent {
    snake: Entity,
}

fn eat_food(
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

fn snake_growth(
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

fn wrapping_edges(mut positions: Query<&mut Position>) {
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

struct GameOverEvent {
    snake: Entity,
}

fn snake_eat_snake(
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

fn game_over(
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

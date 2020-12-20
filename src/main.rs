use bevy::prelude::*;
use bevy::render::pass::ClearColor;
use std::time::Duration;

fn main() {
    App::build()
        /* Window initialization */
        .add_resource(WindowDescriptor {
            title: "Snake!".to_string(),
            width: 1200,
            height: 1200,
            ..Default::default()
        })
        /* Resources */
        .add_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_resource(SnakeTimer::default())
        .add_resource(FoodSpawnTimer::default())
        /* Events */
        .add_event::<GrowthEvent>()
        .add_event::<GameOverEvent>()
        /* Startup */
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup")
        .add_startup_system_to_stage("game_setup", spawn_snake.system())
        /* Systems */
        .add_system(position_translation.system())
        .add_system(sprite_scaling.system())
        .add_system(snake_timer.system())
        .add_system(snake_movement.system())
        .add_system(food_timer.system())
        .add_system(spawn_food.system())
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

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn(Camera2dComponents::default());
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

fn spawn_snake(mut commands: Commands, materials: Res<Materials>) {
    let snake_head = commands
        .spawn(SpriteComponents {
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
        .spawn(SpriteComponents {
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
        .spawn(SpriteComponents {
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

struct SnakeTimer(Timer);

impl Default for SnakeTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(150), true))
    }
}

fn snake_timer(time: Res<Time>, mut snake_timer: ResMut<SnakeTimer>) {
    snake_timer.0.tick(time.delta_seconds);
}

fn snake_movement(
    keyboard_input: Res<Input<KeyCode>>,
    snake_timer: Res<SnakeTimer>,
    mut snakes: Query<(&mut Snake, &Children)>,
    mut positions: Query<&mut Position>,
) {
    for (mut snake, segments) in snakes.iter_mut() {
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

        if !snake_timer.0.finished {
            continue;
        }

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

struct FoodSpawnTimer(Timer);

impl Default for FoodSpawnTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(1000), true))
    }
}

fn food_timer(time: Res<Time>, mut food_timer: ResMut<FoodSpawnTimer>) {
    food_timer.0.tick(time.delta_seconds);
}

struct Food;

fn spawn_food(
    mut commands: Commands,
    materials: Res<Materials>,
    food_timer: Res<FoodSpawnTimer>,
    q: Query<&Position>,
) {
    if food_timer.0.finished {
        let position = Position {
            x: (rand::random::<f32>() * ARENA_WIDTH as f32) as i32,
            y: (rand::random::<f32>() * ARENA_HEIGHT as f32) as i32,
        };

        //check position
        if q.iter().any(|p| p == &position) {
            return;
        }

        commands
            .spawn(SpriteComponents {
                material: materials.food_material.clone(),
                ..Default::default()
            })
            .with(Food)
            .with(position)
            .with(Size::square(0.8));
    }
}

struct GrowthEvent {
    snake: Entity,
}

fn eat_food(
    mut commands: Commands,
    mut growth_events: ResMut<Events<GrowthEvent>>,
    snakes: Query<With<Snake, (Entity, &Children)>>,
    positions: Query<&Position>,
    food_positions: Query<With<Food, (Entity, &Position)>>,
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
    mut commands: Commands,
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
            .spawn(SpriteComponents {
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
    snakes: Query<With<Snake, (Entity, &Children)>>,
    head_positions: Query<With<SnakeHead, &Position>>,
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
    mut commands: Commands,
    materials: Res<Materials>,
    game_over_events: Res<Events<GameOverEvent>>,
    mut game_over_reader: Local<EventReader<GameOverEvent>>,
    foods: Query<With<Food, Entity>>,
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

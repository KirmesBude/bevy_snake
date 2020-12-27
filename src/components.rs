#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BlockDirection {
    Left,
    Right,
    Up,
    Down,
}

impl BlockDirection {
    pub fn opposite(&self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}
pub struct Snake {
    pub direction: BlockDirection,
    pub last_direction: BlockDirection,
}

pub struct SnakeHead;

pub struct SnakeSegment;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct BlockPosition {
    pub x: i32,
    pub y: i32,
}

pub struct BlockSize {
    pub width: f32,
    pub height: f32,
}

impl BlockSize {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

pub struct Food;

pub struct PauseText;

pub struct MenuButton;

use bevy::prelude::*;

pub struct GrowthEvent {
    pub snake: Entity,
}

pub struct GameOverEvent {
    pub snake: Entity,
}

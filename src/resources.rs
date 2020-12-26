use bevy::prelude::*;

pub struct Materials {
    pub head_material: Handle<ColorMaterial>,
    pub food_material: Handle<ColorMaterial>,
    pub body_material: Handle<ColorMaterial>,
}

pub struct Fonts {
    pub pause_font: Handle<Font>,
}

pub struct GrowthEvent {
    pub snake: Entity,
}

pub struct GameOverEvent {
    pub snake: Entity,
}

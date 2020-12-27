use bevy::prelude::*;

pub struct Materials {
    pub head_material: Handle<ColorMaterial>,
    pub food_material: Handle<ColorMaterial>,
    pub body_material: Handle<ColorMaterial>,
}

pub struct Fonts {
    pub pause_font: Handle<Font>,
}

pub struct ButtonMaterials {
    pub normal: Handle<ColorMaterial>,
    pub hovered: Handle<ColorMaterial>,
    pub pressed: Handle<ColorMaterial>,
}

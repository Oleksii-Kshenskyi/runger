use crate::simulation::players::*;

use bevy::render::color::Color;

// UTILITIES

pub fn percent(value: u8) -> f32 {
    value as f32 * 0.01
}

// BOARD

pub const DEFAULT_GRID_SIZE: u32 = 16;
pub const DEFAULT_TILE_SIZE: f32 = 38.0;
pub fn default_entity_to_tile_ratio() -> f32 {
    percent(80)
}
pub fn default_entity_size() -> f32 {
    DEFAULT_TILE_SIZE * default_entity_to_tile_ratio()
}
pub fn default_tile_margin() -> f32 {
    DEFAULT_TILE_SIZE * percent(15)
}
pub fn default_player_count() -> u32 {
    ((DEFAULT_GRID_SIZE * DEFAULT_GRID_SIZE) as f32 * percent(66)) as u32
}

// SIMULATION

pub const TURNS_PER_GEN: u32 = 300;
pub const SECONDS_PER_TURN: f64 = 0.1;

// FOOD
pub fn default_food_count() -> u32 {
    (default_player_count() as f32 * percent(50)) as u32
}
pub fn default_energy_min() -> u32 {
    TURNS_PER_GEN / 2
}
pub fn default_energy_max() -> u32 {
    (TURNS_PER_GEN as f32 / 1.5) as u32
}
pub fn default_food_value() -> u32 {
    TURNS_PER_GEN / 3 * 2
}
pub fn default_player_food_value() -> u32 {
    default_food_value() / 2 + 1
}

// ACTIONS

pub fn action_cost(action_type: &PlayerActionType) -> u32 {
    match *action_type {
        PlayerActionType::Idle => 1,
        PlayerActionType::Turn(_) => 1,
        PlayerActionType::ScanLOS => 1,
        PlayerActionType::Eat => 2,
        PlayerActionType::Move => 3,
        PlayerActionType::Disengage => 15,
        PlayerActionType::Kill => 50,
    }
}

pub const DISENGAGE_LENGTH: u32 = 3;

// LINE OF SIGHT MECHANICS

pub const DEFAULT_LOS_LENGTH: u32 = 3;
pub const DEFAULT_COLOR_ON_LOS_DETECT: Color = Color::rgb(0.8, 1.0, 1.0);

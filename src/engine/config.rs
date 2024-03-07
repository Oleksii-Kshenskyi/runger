pub fn percent(value: u8) -> f32 {
    value as f32 * 0.01
}

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
    ((DEFAULT_GRID_SIZE * DEFAULT_GRID_SIZE) as f32 * percent(10)) as u32
}
// TODO: config should include a constant that determines the amount of turns a single generation takes
// TODO: should also include a constant that defines the speed (number of frames) of a single turn.
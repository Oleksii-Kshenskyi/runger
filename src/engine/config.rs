pub fn percent(value: u8) -> f32 {
    value as f32 * 0.01
}

pub const DEFAULT_GRID_SIZE: u32 = 16;
pub const DEFAULT_TILE_SIZE: f32 = 35.0;
// pub fn default_entity_to_tile_ratio() -> f32 {
//     percent(80)
// }
// pub fn default_entity_size() -> f32 {
//     DEFAULT_TILE_SIZE * default_entity_to_tile_ratio()
// }
pub fn default_tile_margin() -> f32 {
    DEFAULT_TILE_SIZE * percent(15)
}

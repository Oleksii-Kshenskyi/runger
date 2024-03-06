use bevy::prelude::*;

use crate::engine::config::*;

fn grid_to_world(grid_pos: u32) -> f32 {
    grid_pos as f32 * (DEFAULT_TILE_SIZE + default_tile_margin()) - DEFAULT_GRID_SIZE as f32 * (DEFAULT_TILE_SIZE + default_tile_margin()) / 2.0 + DEFAULT_TILE_SIZE / 2.0
}

fn spawn_board(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let tile_color = Color::rgb(0.5, 0.5, 0.5);
    materials.add(tile_color);

    for x in 0..DEFAULT_GRID_SIZE {
        for y in 0..DEFAULT_GRID_SIZE {
            commands.spawn(
                SpriteBundle {
                    sprite: Sprite {
                        color: tile_color.clone(),
                        custom_size: Some(Vec2::new(DEFAULT_TILE_SIZE, DEFAULT_TILE_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        grid_to_world(x),
                        grid_to_world(y),
                        0.0,
                    ),
                    ..Default::default()
                }
            );
        }
    }
}

pub struct GameBoardPlugin;

impl Plugin for GameBoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_board);
    }
}

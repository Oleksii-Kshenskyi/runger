use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use std::collections::HashMap;

use crate::engine::config::*;
use crate::simulation::players::Player;

use super::random::random_board_pos;

#[derive(Component, Debug, PartialEq, Eq, Hash)]
pub struct BoardPosition {
    x: u32,
    y: u32,
}

#[derive(Component, Debug)]
pub enum OccupantType {
    Empty,
    Player(Entity),
}

#[derive(Component, Debug)]
pub struct BoardTile;

impl BoardPosition {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub fn from_tuple((x, y): (u32, u32)) -> Self {
        Self { x, y }
    }
}

#[derive(Bundle)]
pub struct BoardTileBundle {
    pub pos: BoardPosition,
    pub occupant: OccupantType,
    pub sprite: SpriteBundle,
}

#[derive(Resource)]
struct BoardState {
    tiles: HashMap<BoardPosition, Entity>,
}

impl BoardState {
    pub fn new() -> Self {
        Self {
            tiles: HashMap::new(),
        }
    }
}

fn grid_to_world(grid_pos: u32) -> f32 {
    grid_pos as f32 * (DEFAULT_TILE_SIZE + default_tile_margin())
        - DEFAULT_GRID_SIZE as f32 * (DEFAULT_TILE_SIZE + default_tile_margin()) / 2.0
        + DEFAULT_TILE_SIZE / 2.0
}

fn spawn_board(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut board_state: ResMut<BoardState>,
) {
    let tile_color = Color::rgb(0.5, 0.5, 0.5);
    materials.add(tile_color);

    for x in 0..DEFAULT_GRID_SIZE {
        for y in 0..DEFAULT_GRID_SIZE {
            let tile_entity = commands
                .spawn((
                    BoardTileBundle {
                        pos: BoardPosition::new(x, y),
                        occupant: OccupantType::Empty,
                        sprite: SpriteBundle {
                            sprite: Sprite {
                                color: tile_color,
                                custom_size: Some(Vec2::new(DEFAULT_TILE_SIZE, DEFAULT_TILE_SIZE)),
                                ..default()
                            },
                            transform: Transform::from_xyz(grid_to_world(x), grid_to_world(y), 0.0),
                            ..Default::default()
                        },
                    },
                    BoardTile,
                ))
                .id();
            board_state
                .tiles
                .insert(BoardPosition::new(x, y), tile_entity);
        }
    }
}

fn spawn_players(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    board_state: Res<BoardState>,
    mut query: Query<&mut OccupantType, With<BoardTile>>,
) {
    for _ in 0..default_player_count() {
        loop {
            let random_pos = BoardPosition::from_tuple(random_board_pos());
            let tile_entity = *board_state.tiles.get(&random_pos).unwrap();
            if let Ok(mut occupant) = query.get_mut(tile_entity) {
                let triangle = Mesh2dHandle(meshes.add(Triangle2d::new(
                    Vec2::new(-default_entity_size() / 2., default_entity_size() / 2.),
                    Vec2::new(default_entity_size() / 2., 0.),
                    Vec2::new(-default_entity_size() / 2., -default_entity_size() / 2.),
                )));
                *occupant = OccupantType::Player(
                    commands
                        .spawn((
                            MaterialMesh2dBundle {
                                mesh: triangle,
                                material: materials.add(Color::rgb(1.0, 0.0, 0.0)),
                                transform: Transform::from_xyz(
                                    grid_to_world(random_pos.x),
                                    grid_to_world(random_pos.y),
                                    0.1,
                                ),
                                ..default()
                            },
                            Player,
                        ))
                        .id(),
                );
                break;
            }
        }
    }
}

pub struct GameBoardPlugin;

impl Plugin for GameBoardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BoardState::new())
            .add_systems(Startup, (spawn_board, spawn_players).chain());
    }
}

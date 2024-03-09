use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use std::collections::HashMap;

use crate::engine::config::*;
use crate::simulation::players::position_after_turn;
use crate::simulation::players::FacingDirection;
use crate::simulation::players::Player;
use crate::simulation::players::PlayerActionType;

use super::common::triangle_facing;
use super::random::random_board_pos;
use super::random::random_player_action;

#[derive(Component, Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct BoardPosition {
    x: u32,
    y: u32,
}

#[derive(Component, Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum OccupantType {
    Empty,
    Player(Entity),
}

#[derive(Component, Debug, Copy, Clone)]
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

#[derive(Bundle)]
pub struct PlayerBundle {
    pub board_pos: BoardPosition,
    pub is_facing: FacingDirection,
    pub sprite: MaterialMesh2dBundle<ColorMaterial>,
}

#[derive(Resource)]
struct BoardState {
    tiles: HashMap<BoardPosition, Entity>,
}

#[derive(Resource)]
struct Turn {
    num: u32,
}

impl Turn {
    pub fn new() -> Self {
        Self { num: 1 }
    }
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

//  TODO: there should be an update system that advances the board by 1 turn;
// -- This could include stuff like players choosing which actions to take,
// -- players moving, turning, etc.
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
                            PlayerBundle {
                                board_pos: random_pos,
                                is_facing: FacingDirection::Right,
                                sprite: MaterialMesh2dBundle {
                                    mesh: triangle,
                                    material: materials.add(Color::rgb(1.0, 0.0, 0.0)),
                                    transform: Transform::from_xyz(
                                        grid_to_world(random_pos.x),
                                        grid_to_world(random_pos.y),
                                        0.1,
                                    ),
                                    ..default()
                                },
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

fn turn_player(
    direction: &mut FacingDirection,
    mesh: &mut Mesh2dHandle,
    meshes: &mut ResMut<Assets<Mesh>>,
    turn_direction: FacingDirection,
) {
    *direction = position_after_turn(direction, turn_direction).unwrap();
    *mesh = triangle_facing(direction, meshes);
}

fn move_player(
    is_facing: &FacingDirection,
    tile_query: &mut Query<(&mut OccupantType, &BoardPosition), With<BoardTile>>,
    player_pos: &mut BoardPosition,
    player_transform: &mut Transform,
    board_state: &Res<BoardState>,
) {
    let new_pos: (i32, i32) = match (&player_pos, is_facing) {
        (BoardPosition { x, y }, FacingDirection::Up) => (*x as i32, (*y + 1) as i32),
        (BoardPosition { x, y }, FacingDirection::Right) => ((*x + 1) as i32, *y as i32),
        (BoardPosition { x, y }, FacingDirection::Down) => (*x as i32, (*y - 1) as i32),
        (BoardPosition { x, y }, FacingDirection::Left) => ((*x - 1) as i32, *y as i32),
    };
    let within_bounds = new_pos.0 >= 0
        && new_pos.1 >= 0
        && new_pos.0 < DEFAULT_GRID_SIZE as i32
        && new_pos.1 < DEFAULT_GRID_SIZE as i32;
    let maybe_new_entity = if within_bounds {
        board_state
            .tiles
            .get(&BoardPosition::new(new_pos.0 as u32, new_pos.1 as u32))
    } else {
        None
    };
    let allowed_to_move = within_bounds
        && maybe_new_entity.is_some()
        && *tile_query
            .get(*maybe_new_entity.unwrap())
            .as_ref()
            .unwrap()
            .0
            == OccupantType::Empty; // if new (destination) tile's occupant type is empty
    if !allowed_to_move {
        return;
    }

    let new_tile_id = *maybe_new_entity.unwrap();
    let old_tile_id = *board_state.tiles.get(player_pos).unwrap();
    // extract old tile's old occupant type (Player(Entity))
    let old_tile_occ = { *tile_query.get_mut(old_tile_id).unwrap().0 };
    // new tile occ = Player(Entity)
    if let Ok((mut new_tile_occ, _)) = tile_query.get_mut(new_tile_id) {
        *new_tile_occ = old_tile_occ;
    }
    // old tile occ = empty
    if let Ok((mut old_tile_occ, _)) = tile_query.get_mut(old_tile_id) {
        *old_tile_occ = OccupantType::Empty;
    }
    // redraw player (transform)
    *player_transform = Transform::from_xyz(
        grid_to_world(new_pos.0 as u32),
        grid_to_world(new_pos.1 as u32),
        0.1,
    );
    // update new player board position
    *player_pos = BoardPosition {
        x: new_pos.0 as u32,
        y: new_pos.1 as u32,
    };
}

fn simulation_ongoing(turn: Res<Turn>) -> bool {
    turn.num <= TURNS_PER_GEN
}

fn advance_players(
    mut meshes: ResMut<Assets<Mesh>>,
    mut turn: ResMut<Turn>,
    board_state: Res<BoardState>,
    mut tile_query: Query<(&mut OccupantType, &BoardPosition), With<BoardTile>>,
    mut player_query: Query<
        (
            &mut BoardPosition,
            &mut FacingDirection,
            &mut Mesh2dHandle,
            &mut Transform,
        ),
        With<Player>,
    >,
) {
    for (mut pos, mut direction, mut mesh, mut transform) in player_query.iter_mut() {
        match random_player_action() {
            PlayerActionType::Idle => (),
            PlayerActionType::Move => move_player(
                &direction,
                &mut tile_query,
                &mut pos,
                &mut transform,
                &board_state,
            ),
            PlayerActionType::Turn(FacingDirection::Right) => turn_player(
                &mut direction,
                &mut mesh,
                &mut meshes,
                FacingDirection::Right,
            ),
            PlayerActionType::Turn(FacingDirection::Left) => turn_player(
                &mut direction,
                &mut mesh,
                &mut meshes,
                FacingDirection::Left,
            ),
            act => unreachable!(
                "Incorrect action type while trying to advance players: {:#?}",
                act
            ),
        }
    }

    turn.num += 1;
}

pub struct GameBoardPlugin;

impl Plugin for GameBoardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BoardState::new())
            .insert_resource(Time::<Fixed>::from_seconds(SECONDS_PER_TURN))
            .insert_resource(Turn::new())
            .add_systems(Startup, (spawn_board, spawn_players).chain())
            .add_systems(FixedUpdate, advance_players.run_if(simulation_ongoing));
    }
}

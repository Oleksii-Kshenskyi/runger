use std::{collections::HashMap, error::Error, f32::consts::PI, fmt::Display};

use bevy::prelude::*;

use crate::engine::config::*;
use crate::simulation::players::{position_after_turn, FacingDirection};

#[derive(Debug)]
pub struct RungerError {
    message: String,
}

pub fn rerror(msg: &'static str) -> Box<dyn Error> {
    Box::new(RungerError {
        message: msg.to_string(),
    })
}

impl Display for RungerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for RungerError {}

#[derive(Component, Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct BoardPosition {
    pub x: u32,
    pub y: u32,
}

impl BoardPosition {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub fn from_tuple((x, y): (u32, u32)) -> Self {
        Self { x, y }
    }
}

#[derive(Component, Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum OccupantType {
    Empty,
    Player(Entity),
    Food(Entity),
}

#[derive(Resource)]
pub struct BoardState {
    pub tiles: HashMap<BoardPosition, Entity>,
}

#[derive(Component, Debug, Copy, Clone)]
pub struct BoardTile;

pub fn turn_right(is_facing: &mut FacingDirection, transform: &mut Transform) {
    *is_facing = position_after_turn(is_facing, FacingDirection::Right).unwrap();
    transform.rotate_z(-PI / 2.);
}

pub fn turn_left(is_facing: &mut FacingDirection, transform: &mut Transform) {
    *is_facing = position_after_turn(is_facing, FacingDirection::Left).unwrap();
    transform.rotate_z(PI / 2.);
}

pub fn predict_move_pos(player_pos: &BoardPosition, is_facing: &FacingDirection) -> (i32, i32) {
    match (&player_pos, is_facing) {
        (BoardPosition { x, y }, FacingDirection::Up) => (*x as i32, (*y + 1) as i32),
        (BoardPosition { x, y }, FacingDirection::Right) => ((*x + 1) as i32, *y as i32),
        (BoardPosition { x, y }, FacingDirection::Down) => (*x as i32, (*y - 1) as i32),
        (BoardPosition { x, y }, FacingDirection::Left) => ((*x - 1) as i32, *y as i32),
    }
}

pub fn pos_within_bounds(pos_to_check: &(i32, i32)) -> bool {
    pos_to_check.0 >= 0
        && pos_to_check.1 >= 0
        && pos_to_check.0 < DEFAULT_GRID_SIZE as i32
        && pos_to_check.1 < DEFAULT_GRID_SIZE as i32
}

pub fn tile_entity_by_pos<'a>(
    new_pos: (i32, i32),
    board_state: &'a Res<BoardState>,
) -> Option<&'a Entity> {
    let within_bounds = pos_within_bounds(&new_pos);
    if within_bounds {
        board_state
            .tiles
            .get(&BoardPosition::new(new_pos.0 as u32, new_pos.1 as u32))
    } else {
        None
    }
}

pub fn player_can_move_here(
    tile_entity: &Entity,
    tile_query: &mut Query<&mut OccupantType, With<BoardTile>>,
) -> bool {
    *tile_query.get(*tile_entity).unwrap() == OccupantType::Empty
    // if new (destination) tile's occupant type is empty
}

use std::{collections::HashMap, error::Error, fmt::Display};

use bevy::{prelude::*, sprite::Mesh2dHandle};

use crate::engine::config::*;
use crate::simulation::players::FacingDirection;

use super::config::default_entity_size;

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

pub fn triangle_facing(
    direction: &FacingDirection,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> Mesh2dHandle {
    match *direction {
        FacingDirection::Up => Mesh2dHandle(meshes.add(Triangle2d::new(
            Vec2::new(0., default_entity_size() / 2.),
            Vec2::new(-default_entity_size() / 2., -default_entity_size() / 2.),
            Vec2::new(default_entity_size() / 2., -default_entity_size() / 2.),
        ))),
        FacingDirection::Right => Mesh2dHandle(meshes.add(Triangle2d::new(
            Vec2::new(-default_entity_size() / 2., default_entity_size() / 2.),
            Vec2::new(default_entity_size() / 2., 0.),
            Vec2::new(-default_entity_size() / 2., -default_entity_size() / 2.),
        ))),
        FacingDirection::Down => Mesh2dHandle(meshes.add(Triangle2d::new(
            Vec2::new(-default_entity_size() / 2., default_entity_size() / 2.),
            Vec2::new(default_entity_size() / 2., default_entity_size() / 2.),
            Vec2::new(0., -default_entity_size() / 2.),
        ))),
        FacingDirection::Left => Mesh2dHandle(meshes.add(Triangle2d::new(
            Vec2::new(-default_entity_size() / 2., 0.),
            Vec2::new(default_entity_size() / 2., default_entity_size() / 2.),
            Vec2::new(default_entity_size() / 2., -default_entity_size() / 2.),
        ))),
    }
}

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
}

#[derive(Resource)]
pub struct BoardState {
    pub tiles: HashMap<BoardPosition, Entity>,
}

#[derive(Component, Debug, Copy, Clone)]
pub struct BoardTile;

pub fn predict_move_pos(player_pos: &BoardPosition, is_facing: &FacingDirection) -> (i32, i32) {
    match (&player_pos, is_facing) {
        (BoardPosition { x, y }, FacingDirection::Up) => (*x as i32, (*y + 1) as i32),
        (BoardPosition { x, y }, FacingDirection::Right) => ((*x + 1) as i32, *y as i32),
        (BoardPosition { x, y }, FacingDirection::Down) => (*x as i32, (*y - 1) as i32),
        (BoardPosition { x, y }, FacingDirection::Left) => ((*x - 1) as i32, *y as i32),
    }
}

pub fn tile_entity_by_pos<'a>(
    new_pos: (i32, i32),
    board_state: &'a Res<BoardState>,
) -> Option<&'a Entity> {
    let within_bounds = new_pos.0 >= 0
        && new_pos.1 >= 0
        && new_pos.0 < DEFAULT_GRID_SIZE as i32
        && new_pos.1 < DEFAULT_GRID_SIZE as i32;
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

use crate::engine::common::*;

use bevy::prelude::*;

use std::error::Error;

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug)]
pub struct Food;

#[derive(Component, Debug, Clone, Copy)]
pub enum FacingDirection {
    Up,
    Left,
    Down,
    Right,
}

#[derive(Component, Debug)]
pub enum PlayerActionType {
    Idle,
    Move,
    Turn(FacingDirection),
    Eat,
    Kill,
}

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlayerStatus {
    Alive,
    DedPepega,
}

#[derive(Component, Debug)]
pub struct Hunger {
    pub value: u32,
}

impl Hunger {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

#[derive(Component, Debug)]
pub struct Vitals {
    pub hunger: Hunger,
    pub status: PlayerStatus,
}

impl Vitals {
    pub fn new(hunger: u32) -> Self {
        Self {
            hunger: Hunger::new(hunger),
            status: PlayerStatus::Alive,
        }
    }
}

pub fn position_after_turn(
    is: &FacingDirection,
    turn_direction: FacingDirection,
) -> Result<FacingDirection, Box<dyn Error>> {
    match turn_direction {
        FacingDirection::Up => Err(rerror("Player tried to turn 'up', which is impossible")),
        FacingDirection::Down => Err(rerror("Player tried to turn 'down', which is impossible")),
        FacingDirection::Left => match is {
            FacingDirection::Up => Ok(FacingDirection::Left),
            FacingDirection::Right => Ok(FacingDirection::Up),
            FacingDirection::Down => Ok(FacingDirection::Right),
            FacingDirection::Left => Ok(FacingDirection::Down),
        },
        FacingDirection::Right => match is {
            FacingDirection::Up => Ok(FacingDirection::Right),
            FacingDirection::Right => Ok(FacingDirection::Down),
            FacingDirection::Down => Ok(FacingDirection::Left),
            FacingDirection::Left => Ok(FacingDirection::Up),
        },
    }
}

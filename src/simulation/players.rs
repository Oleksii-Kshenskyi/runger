use crate::engine::common::*;

use bevy::prelude::*;

use std::error::Error;

// TODO: Each turn, the player decides (either randomly or based on its "brain") which action from the types of actions available to take
#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug)]
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

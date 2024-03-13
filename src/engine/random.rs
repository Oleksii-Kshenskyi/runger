use rand::{thread_rng, Rng};

use crate::{
    engine::config::*,
    simulation::players::{FacingDirection, PlayerActionType},
};

pub fn random_board_pos() -> (u32, u32) {
    let mut rng = thread_rng();

    (
        rng.gen_range(0..DEFAULT_GRID_SIZE),
        rng.gen_range(0..DEFAULT_GRID_SIZE),
    )
}

pub fn random_player_action() -> PlayerActionType {
    let mut rng = thread_rng();

    let action_num = rng.gen_range(0..6);
    match action_num {
        0 => PlayerActionType::Idle,
        1 => PlayerActionType::Move,
        2 => PlayerActionType::Turn(FacingDirection::Left),
        3 => PlayerActionType::Turn(FacingDirection::Right),
        4 => PlayerActionType::Eat,
        5 => PlayerActionType::Kill,
        _ => unreachable!("{} is not allowed in random_action_type()", action_num),
    }
}

pub fn random_hunger_start() -> u32 {
    let mut rng = thread_rng();
    rng.gen_range(default_hunger_min()..=default_hunger_max())
}

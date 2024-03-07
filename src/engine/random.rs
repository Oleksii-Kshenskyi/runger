use rand::{thread_rng, Rng};

use crate::engine::config::*;

pub fn random_board_pos() -> (u32, u32) {
    let mut rng = thread_rng();

    (
        rng.gen_range(0..DEFAULT_GRID_SIZE),
        rng.gen_range(0..DEFAULT_GRID_SIZE),
    )
}

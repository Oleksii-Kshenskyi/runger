use std::collections::HashMap;
use crate::engine::common::BoardPosition;
use crate::simulation::players::Player;

use bevy::prelude::*;

/// Debug system for logging if there are multiple players occupying the same tile
fn log_positioning_conflicts(player_query: Query<&BoardPosition, With<Player>>) {
    let mut conflict_map: HashMap<BoardPosition, u8> = HashMap::new();
    for player_pos in player_query.iter() {
        if !conflict_map.contains_key(player_pos) {
            conflict_map.insert(*player_pos, 1);
        } else {
            let num_players = conflict_map.get(player_pos).unwrap() + 1;
            conflict_map.insert(*player_pos, num_players);
            error!(name: "kek", "Position {:#?} contains {} players!!", player_pos, num_players);
        }
    }
}
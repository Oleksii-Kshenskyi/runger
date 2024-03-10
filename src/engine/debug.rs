use std::collections::HashMap;
use crate::engine::common::*;
use crate::simulation::players::{Player, FacingDirection};

use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;

use crate::engine::config::default_entity_size;

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

// A function for producing a triangle mesh facing one of four cardinal directions
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
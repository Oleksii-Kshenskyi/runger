use std::{collections::HashMap, error::Error, fmt::Display};

use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

use crate::engine::config::*;
use crate::simulation::players::{FacingDirection, Food, Hunger};

#[derive(Debug)]
pub struct RungerError {
    message: String,
}

pub fn rerror(msg: &str) -> Box<dyn Error> {
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
pub struct Board {
    occupants: HashMap<BoardPosition, OccupantType>,
}

impl Board {
    fn looking_pos(player_pos: &BoardPosition, is_facing: &FacingDirection) -> (i32, i32) {
        match (&player_pos, is_facing) {
            (BoardPosition { x, y }, FacingDirection::Up) => (*x as i32, (*y as i32 + 1)),
            (BoardPosition { x, y }, FacingDirection::Right) => ((*x as i32 + 1), *y as i32),
            (BoardPosition { x, y }, FacingDirection::Down) => (*x as i32, (*y as i32 - 1)),
            (BoardPosition { x, y }, FacingDirection::Left) => ((*x as i32 - 1), *y as i32),
        }
    }

    fn pos_within_bounds(pos_to_check: &(i32, i32)) -> bool {
        pos_to_check.0 >= 0
            && pos_to_check.1 >= 0
            && pos_to_check.0 < DEFAULT_GRID_SIZE as i32
            && pos_to_check.1 < DEFAULT_GRID_SIZE as i32
    }

    pub fn new() -> Self {
        Self {
            occupants: HashMap::new(),
        }
    }

    pub fn occ_at(&self, pos: &BoardPosition) -> Option<&OccupantType> {
        self.occupants.get(pos)
    }

    pub fn occ_at_mut(&mut self, pos: &BoardPosition) -> Option<&mut OccupantType> {
        self.occupants.get_mut(pos)
    }

    pub fn looking_at(
        &self,
        looker_pos: &BoardPosition,
        looker_facing: &FacingDirection,
    ) -> Option<(BoardPosition, &OccupantType)> {
        let looking_pos = Board::looking_pos(looker_pos, looker_facing);
        if Self::pos_within_bounds(&looking_pos) {
            let pos = BoardPosition::new(looking_pos.0 as u32, looking_pos.1 as u32);
            self.occupants.get(&pos).map(|o| (pos, o))
        } else {
            None
        }
    }

    pub fn looking_at_mut(
        &mut self,
        looker_pos: &BoardPosition,
        looker_facing: &FacingDirection,
    ) -> Option<(BoardPosition, &mut OccupantType)> {
        let looking_pos = Board::looking_pos(looker_pos, looker_facing);
        if Self::pos_within_bounds(&looking_pos) {
            let pos = BoardPosition::new(looking_pos.0 as u32, looking_pos.1 as u32);
            self.occupants.get_mut(&pos).map(|o| (pos, o))
        } else {
            None
        }
    }

    pub fn add_occ(&mut self, pos: BoardPosition, occ: OccupantType) {
        self.occupants.insert(pos, occ);
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct BoardTile;

#[derive(Debug)]
pub enum FoodType {
    Meal,
    DeadMeat,
}

#[derive(Bundle)]
pub struct FoodBundle {
    pub board_pos: BoardPosition,
    pub energy_value: Hunger,
    pub sprite: MaterialMesh2dBundle<ColorMaterial>,
}

#[derive(Default, Clone, Copy, States, Debug, Hash, PartialEq, Eq)]
pub enum VisualizerState {
    #[default]
    SimulationRunning,
    GenerationFinished,
}

pub fn grid_to_world(grid_pos: u32) -> f32 {
    grid_pos as f32 * (DEFAULT_TILE_SIZE + default_tile_margin())
        - DEFAULT_GRID_SIZE as f32 * (DEFAULT_TILE_SIZE + default_tile_margin()) / 2.0
        + DEFAULT_TILE_SIZE / 2.0
}

pub fn place_food_at(
    commands: &mut Commands,
    pos: BoardPosition,
    food_type: FoodType,
    board: &mut ResMut<Board>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> Result<(), Box<dyn Error>> {
    let (energy_value, food_color) = match food_type {
        FoodType::Meal => (default_food_value(), Color::rgb(1., 0.5, 0.)),
        FoodType::DeadMeat => (default_player_food_value(), Color::rgb(0., 0., 0.)),
    };

    let mesh = meshes.add(Circle {
        radius: default_entity_size() / 2.,
    });
    if let Some(occupant) = board.occ_at_mut(&pos) {
        if *occupant != OccupantType::Empty {
            return Err(rerror("Trying to place food on a non-empty tile!"));
        };
        *occupant = OccupantType::Food(
            commands
                .spawn((
                    FoodBundle {
                        energy_value: Hunger::new(energy_value),
                        board_pos: pos,
                        sprite: MaterialMesh2dBundle {
                            mesh: Mesh2dHandle(mesh),
                            material: materials.add(food_color),
                            transform: Transform::from_xyz(
                                grid_to_world(pos.x),
                                grid_to_world(pos.y),
                                0.9,
                            ),
                            ..default()
                        },
                    },
                    Food,
                ))
                .id(),
        );
        return Ok(());
    }

    Err(rerror(&format!(
        "place_food_at(): no entry on occupant at {:?}...",
        &pos
    )))
}

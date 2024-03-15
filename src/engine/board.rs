use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

use std::f32::consts::PI;

use crate::{
    engine::common::*, engine::config::*, engine::random::*,
    simulation::players::*,
};

#[derive(Bundle)]
pub struct BoardTileBundle {
    pub sprite: SpriteBundle,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub board_pos: BoardPosition,
    pub is_facing: FacingDirection,
    pub vitals: Vitals,
    pub sprite: MaterialMesh2dBundle<ColorMaterial>,
}

#[derive(Resource)]
struct Turn {
    num: u32,
}

impl Turn {
    pub fn new() -> Self {
        Self { num: 0 }
    }
}

fn spawn_board(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut board: ResMut<Board>,
) {
    let tile_color = Color::rgb(0.5, 0.5, 0.5);
    materials.add(tile_color);

    for x in 0..DEFAULT_GRID_SIZE {
        for y in 0..DEFAULT_GRID_SIZE {
            commands.spawn((
                BoardTileBundle {
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
            ));
            board.add_occ(BoardPosition::new(x, y), OccupantType::Empty);
        }
    }
}

fn spawn_players(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut board: ResMut<Board>,
) {
    for _ in 0..default_player_count() {
        loop {
            let random_pos = BoardPosition::from_tuple(random_board_pos());

            if let Some(occupant) = board.occ_at_mut(&random_pos) {
                if *occupant != OccupantType::Empty {
                    continue;
                }
                let triangle = Mesh2dHandle(meshes.add(Triangle2d::new(
                    Vec2::new(0., default_entity_size() / 2.),
                    Vec2::new(-default_entity_size() / 2., -default_entity_size() / 2.),
                    Vec2::new(default_entity_size() / 2., -default_entity_size() / 2.),
                )));
                *occupant = OccupantType::Player(
                    commands
                        .spawn((
                            PlayerBundle {
                                board_pos: random_pos,
                                is_facing: FacingDirection::Right,
                                vitals: Vitals::new(random_hunger_start()),
                                sprite: MaterialMesh2dBundle {
                                    mesh: triangle,
                                    material: materials.add(Color::rgb(1.0, 0.0, 0.0)),
                                    transform: Transform::from_xyz(
                                        grid_to_world(random_pos.x),
                                        grid_to_world(random_pos.y),
                                        1.0,
                                    )
                                    .with_rotation(Quat::from_rotation_z(-PI / 2.0)),
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

fn spawn_food(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut board: ResMut<Board>,
) {
    for _ in 0..default_food_count() {
        loop {
            let random_pos = BoardPosition::from_tuple(random_board_pos());
            if let Some(occ) = board.occ_at(&random_pos) {
                if *occ != OccupantType::Empty {
                    continue;
                }

                place_food_at(
                    &mut commands,
                    random_pos,
                    FoodType::Meal,
                    &mut board,
                    &mut materials,
                    &mut meshes,
                )
                .unwrap();
                break;
            }
        }
    }
}

fn advance_turn(mut turn: ResMut<Turn>, mut states: ResMut<NextState<VisualizerState>>) {
    turn.num += 1;

    if turn.num > TURNS_PER_GEN {
        states.set(VisualizerState::GenerationFinished);
    }
}

fn log_survival_rate(player_query: Query<&Vitals, With<Player>>) {
    let mut survived: u32 = 0;
    for vitals in player_query.iter() {
        match vitals.status {
            PlayerStatus::Alive => survived += 1,
            PlayerStatus::DedPepega => (),
        }
    }
    warn!(
        "Simulation over! Started with {} players. Survived: {} players, died: {} players. Survival rate: {:.2}%.",
        default_player_count(),
        survived,
        default_player_count() - survived,
        (survived as f32 / default_player_count() as f32) * 100.
    );
}

pub struct GameBoardPlugin;

impl Plugin for GameBoardPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<VisualizerState>()
            .insert_resource(Board::new())
            .insert_resource(Time::<Fixed>::from_seconds(SECONDS_PER_TURN))
            .insert_resource(Turn::new())
            .add_systems(Startup, (spawn_board, spawn_players, spawn_food).chain())
            .add_systems(
                FixedUpdate,
                (advance_turn).run_if(in_state(VisualizerState::SimulationRunning)),
            )
            .add_systems(
                OnEnter(VisualizerState::GenerationFinished),
                log_survival_rate,
            );
    }
}

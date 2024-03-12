use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

use std::collections::HashMap;
use std::f32::consts::PI;

use crate::{engine::common::*, engine::config::*, engine::random::*, simulation::players::*};

#[derive(Bundle)]
pub struct BoardTileBundle {
    pub pos: BoardPosition,
    pub occupant: OccupantType,
    pub sprite: SpriteBundle,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub board_pos: BoardPosition,
    pub is_facing: FacingDirection,
    pub vitals: Vitals,
    pub sprite: MaterialMesh2dBundle<ColorMaterial>,
}

#[derive(Bundle)]
pub struct FoodBundle {
    pub board_pos: BoardPosition,
    pub energy_value: Hunger,
    pub sprite: MaterialMesh2dBundle<ColorMaterial>,
}

#[derive(Resource)]
struct Turn {
    num: u32,
}

impl Turn {
    pub fn new() -> Self {
        Self { num: 1 }
    }
}

#[derive(Resource)]
struct GameOver(bool);

impl BoardState {
    pub fn new() -> Self {
        Self {
            tiles: HashMap::new(),
        }
    }
}

fn grid_to_world(grid_pos: u32) -> f32 {
    grid_pos as f32 * (DEFAULT_TILE_SIZE + default_tile_margin())
        - DEFAULT_GRID_SIZE as f32 * (DEFAULT_TILE_SIZE + default_tile_margin()) / 2.0
        + DEFAULT_TILE_SIZE / 2.0
}

fn spawn_board(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut board_state: ResMut<BoardState>,
) {
    let tile_color = Color::rgb(0.5, 0.5, 0.5);
    materials.add(tile_color);

    for x in 0..DEFAULT_GRID_SIZE {
        for y in 0..DEFAULT_GRID_SIZE {
            let tile_entity = commands
                .spawn((
                    BoardTileBundle {
                        pos: BoardPosition::new(x, y),
                        occupant: OccupantType::Empty,
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
                ))
                .id();
            board_state
                .tiles
                .insert(BoardPosition::new(x, y), tile_entity);
        }
    }
}

fn spawn_players(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    board_state: Res<BoardState>,
    mut query: Query<&mut OccupantType, With<BoardTile>>,
) {
    for _ in 0..default_player_count() {
        loop {
            let random_pos = BoardPosition::from_tuple(random_board_pos());
            let tile_entity = *board_state.tiles.get(&random_pos).unwrap();
            if let Ok(mut occupant) = query.get_mut(tile_entity) {
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
    mut tile_query: Query<&mut OccupantType, With<BoardTile>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    board_state: Res<BoardState>,
) {
    for _ in 0..default_food_count() {
        loop {
            let random_pos = BoardPosition::from_tuple(random_board_pos());
            if let Some(tile_id) = board_state.tiles.get(&random_pos) {
                {
                    let occ = tile_query.get(*tile_id);
                    if occ.is_err() || *occ.unwrap() != OccupantType::Empty {
                        continue;
                    }
                }

                let occupant = tile_query.get_mut(*tile_id);
                let mesh = meshes.add(Circle {
                    radius: default_entity_size() / 2.,
                });
                let mut occupant = occupant.unwrap();
                *occupant = OccupantType::Food(
                    commands
                        .spawn((
                            FoodBundle {
                                energy_value: Hunger::new(default_food_value()),
                                board_pos: random_pos,
                                sprite: MaterialMesh2dBundle {
                                    mesh: Mesh2dHandle(mesh),
                                    material: materials.add(Color::rgb(1., 0.5, 0.)),
                                    transform: Transform::from_xyz(
                                        grid_to_world(random_pos.x),
                                        grid_to_world(random_pos.y),
                                        0.9,
                                    ),
                                    ..default()
                                },
                            },
                            Food,
                        ))
                        .id(),
                );
                break;
            }
        }
    }
}

fn move_player(
    is_facing: &FacingDirection,
    tile_query: &mut Query<&mut OccupantType, With<BoardTile>>,
    player_pos: &mut BoardPosition,
    player_transform: &mut Transform,
    board_state: &Res<BoardState>,
) {
    let new_pos: (i32, i32) = predict_move_pos(player_pos, is_facing);
    let maybe_new_tile = tile_entity_by_pos(new_pos, board_state);

    if maybe_new_tile.is_none() || !player_can_move_here(maybe_new_tile.unwrap(), tile_query) {
        return;
    }

    let new_tile_id = *maybe_new_tile.unwrap();
    let old_tile_id = *board_state.tiles.get(player_pos).unwrap();
    // extract old tile's old occupant type (Player(Entity))
    let old_tile_occ = { *tile_query.get_mut(old_tile_id).unwrap() };
    // new tile occ = Player(Entity)
    if let Ok(mut new_tile_occ) = tile_query.get_mut(new_tile_id) {
        *new_tile_occ = old_tile_occ;
    }
    // old tile occ = empty
    if let Ok(mut old_tile_occ) = tile_query.get_mut(old_tile_id) {
        *old_tile_occ = OccupantType::Empty;
    }
    // move player (transform)
    player_transform.translation = Vec3::new(
        grid_to_world(new_pos.0 as u32),
        grid_to_world(new_pos.1 as u32),
        0.1,
    );
    // update new player board position
    *player_pos = BoardPosition {
        x: new_pos.0 as u32,
        y: new_pos.1 as u32,
    };
}

fn simulation_ongoing(turn: Res<Turn>) -> bool {
    turn.num <= TURNS_PER_GEN
}
fn simulation_over_once(turn: Res<Turn>, its_joever: Res<GameOver>) -> bool {
    turn.num > TURNS_PER_GEN && !its_joever.0
}

fn player_eat(
    commands: &mut Commands,
    pos: &BoardPosition,
    direction: &FacingDirection,
    vitals: &mut Vitals,
    board_state: &Res<BoardState>,
    tile_query: &mut Query<&mut OccupantType, With<BoardTile>>,
    food_query: &Query<&Hunger, (With<Food>, Without<Player>)>,
) {
    let food_pos = predict_move_pos(pos, direction);
    if !pos_within_bounds(&food_pos) {
        return;
    }

    let food_tile_id = board_state
        .tiles
        .get(&BoardPosition::new(food_pos.0 as u32, food_pos.1 as u32))
        .copied();
    let occupant = tile_query.get_mut(food_tile_id.unwrap());

    if let Ok(mut occ) = occupant {
        if let OccupantType::Food(food_id) = *occ {
            vitals.hunger.value += food_query.get(food_id).unwrap().value;
            commands.entity(food_id).despawn_recursive();
            *occ = OccupantType::Empty;
        }
    }
}

fn update_player_vitals(
    vitals: &mut Vitals,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    player_mat: &mut Handle<ColorMaterial>,
) {
    vitals.hunger.value -= 1;
    if vitals.hunger.value == 0 {
        vitals.status = PlayerStatus::DedPepega;
        *player_mat = materials.add(Color::rgb(0., 0., 0.));
    }
}

fn player_kill(
    mut commands: &mut Commands,
    board_state: &Res<BoardState>,
    mut materials: &mut ResMut<Assets<ColorMaterial>>,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    actor_id: Entity,
    tile_query: &mut Query<&mut OccupantType, With<BoardTile>>,
    mut player_query: &mut Query<
        (
            Entity,
            &mut BoardPosition,
            &mut FacingDirection,
            &mut Transform,
            &mut Vitals,
            &mut Handle<ColorMaterial>,
            &mut Mesh2dHandle,
        ),
        With<Player>,
    >,
    food_query: &Query<&Hunger, (With<Food>, Without<Player>)>,
) {
    let (_, actor_pos, actor_facing, _, _, _, _) = player_query.get(actor_id).unwrap();
    let victim_pos = predict_move_pos(actor_pos, actor_facing);
    if !pos_within_bounds(&victim_pos) {
        return;
    }

    let victim_tile_id = *tile_entity_by_pos(victim_pos, &board_state).unwrap();
    let mut victim_tile_occ = tile_query.get_mut(victim_tile_id).unwrap();
    let victim_id = match *victim_tile_occ {
        OccupantType::Player(id) => Some(id),
        _ => None,
    };
    if victim_id.is_none() {
        return;
    }
    let victim_id = victim_id.unwrap();

    let (_, _, _, _, mut victim_vitals, mut victim_color, mut victim_mesh) =
        player_query.get_mut(victim_id).unwrap();
    victim_vitals.status = PlayerStatus::DedPepega;
    victim_vitals.hunger.value = default_player_food_value();
    *victim_color = materials.add(Color::rgb(0., 0., 0.));
    *victim_mesh = Mesh2dHandle(meshes.add(Circle::new(default_entity_size() / 2.)));
    commands.entity(victim_id).remove::<Player>().insert(Food);

    *victim_tile_occ = OccupantType::Food(victim_id);
}

fn advance_players(
    mut commands: Commands,
    mut turn: ResMut<Turn>,
    board_state: Res<BoardState>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut tile_query: Query<&mut OccupantType, With<BoardTile>>,
    mut player_query: Query<
        (
            Entity,
            &mut BoardPosition,
            &mut FacingDirection,
            &mut Transform,
            &mut Vitals,
            &mut Handle<ColorMaterial>,
            &mut Mesh2dHandle,
        ),
        With<Player>,
    >,
    food_query: Query<&Hunger, (With<Food>, Without<Player>)>,
) {
    for (
        player_id,
        mut pos,
        mut direction,
        mut transform,
        mut vitals,
        mut player_mat,
        _,
    ) in player_query.iter_mut()
    {
        if vitals.status == PlayerStatus::DedPepega {
            continue;
        }
        match random_player_action() {
            PlayerActionType::Idle => (),
            PlayerActionType::Move => move_player(
                &direction,
                &mut tile_query,
                &mut pos,
                &mut transform,
                &board_state,
            ),
            PlayerActionType::Turn(FacingDirection::Right) => {
                turn_right(&mut direction, &mut transform)
            }
            PlayerActionType::Turn(FacingDirection::Left) => {
                turn_left(&mut direction, &mut transform)
            }
            PlayerActionType::Eat => player_eat(
                &mut commands,
                &pos,
                &direction,
                &mut vitals,
                &board_state,
                &mut tile_query,
                &food_query,
            ),
            PlayerActionType::Kill => player_kill(
                &mut commands,
                &board_state,
                &mut materials,
                &mut meshes,
                player_id,
                &mut tile_query,
                &mut player_query,
                &food_query,
            ),
            act => unreachable!(
                "Incorrect action type while trying to advance players: {:#?}",
                act
            ),
        }
        update_player_vitals(&mut vitals, &mut materials, &mut player_mat);
    }

    turn.num += 1;
}

fn log_survival_rate(player_query: Query<&Vitals, With<Player>>, mut its_joever: ResMut<GameOver>) {
    its_joever.0 = true;
    let (mut survived, mut died): (u32, u32) = (0, 0);
    for vitals in player_query.iter() {
        match vitals.status {
            PlayerStatus::Alive => survived += 1,
            PlayerStatus::DedPepega => died += 1,
        }
    }
    warn!(
        "Simulation over! Survived: {} players, died: {} players. Survival rate: {:.2}%.",
        survived,
        died,
        (survived as f32 / (survived + died) as f32) * 100.
    );
}

pub struct GameBoardPlugin;

impl Plugin for GameBoardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BoardState::new())
            .insert_resource(Time::<Fixed>::from_seconds(SECONDS_PER_TURN))
            .insert_resource(Turn::new())
            .insert_resource(GameOver(false))
            .add_systems(Startup, (spawn_board, spawn_players, spawn_food).chain())
            .add_systems(FixedUpdate, advance_players.run_if(simulation_ongoing))
            .add_systems(FixedLast, log_survival_rate.run_if(simulation_over_once));
    }
}

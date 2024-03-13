use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

use std::collections::HashMap;
use std::error::Error;
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

#[derive(Event, Debug)]
pub struct KillEvent {
    pub killer_id: Entity,
    pub killer_facing: FacingDirection,
}

#[derive(Event, Debug)]
pub struct EatEvent {
    pub gorger_id: Entity,
    pub gorger_facing: FacingDirection,
}

#[derive(Event, Debug)]
pub struct MoveEvent {
    pub mover_id: Entity,
    pub mover_facing: FacingDirection,
}

#[derive(Event, Debug)]
pub struct TurnEvent {
    pub turner_id: Entity,
    pub turner_facing: FacingDirection,
    pub turn_direction: FacingDirection,
}

#[derive(Event, Debug)]
pub struct UpdateVitalsEvent {
    pub hungerer_id: Entity,
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

pub fn place_food_at(
    commands: &mut Commands,
    pos: BoardPosition,
    food_type: FoodType,
    board_state: &Res<BoardState>,
    tile_query: &mut Query<&mut OccupantType, With<BoardTile>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> Result<(), Box<dyn Error>> {
    let (energy_value, food_color) = match food_type {
        FoodType::Meal => (default_food_value(), Color::rgb(1., 0.5, 0.)),
        FoodType::DeadMeat => (default_player_food_value(), Color::rgb(0., 0., 0.)),
    };

    let tile_id = board_state.tiles.get(&pos).unwrap();
    let occupant = tile_query.get_mut(*tile_id);
    let mesh = meshes.add(Circle {
        radius: default_entity_size() / 2.,
    });
    let mut occupant = occupant.unwrap();
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

    Ok(())
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
                let occ = tile_query.get(*tile_id);
                if occ.is_err() || *occ.unwrap() != OccupantType::Empty {
                    continue;
                }

                place_food_at(
                    &mut commands,
                    random_pos,
                    FoodType::Meal,
                    &board_state,
                    &mut tile_query,
                    &mut materials,
                    &mut meshes,
                )
                .unwrap();
                break;
            }
        }
    }
}

fn player_move_listener(
    mut move_events: EventReader<MoveEvent>,
    mut tile_query: Query<&mut OccupantType, With<BoardTile>>,
    mut player_query: Query<
        (&mut BoardPosition, &mut Transform),
        (With<Player>, Without<BoardTile>),
    >,
    board_state: Res<BoardState>,
) {
    for event in move_events.read() {
        let (mover_id, mover_facing) = (event.mover_id, event.mover_facing);
        let maybe_query_res = player_query.get_mut(mover_id);
        if maybe_query_res.is_err() {
            continue;
        }
        let (mut mover_pos, mut mover_transform) = maybe_query_res.unwrap();
        let new_pos: (i32, i32) = predict_move_pos(&mover_pos, &mover_facing);
        let maybe_new_tile = tile_entity_by_pos(new_pos, &board_state);

        if maybe_new_tile.is_none()
            || !player_can_move_here(maybe_new_tile.unwrap(), &mut tile_query)
        {
            return;
        }

        let new_tile_id = *maybe_new_tile.unwrap();
        let old_tile_id = *board_state.tiles.get(&mover_pos).unwrap();
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
        mover_transform.translation = Vec3::new(
            grid_to_world(new_pos.0 as u32),
            grid_to_world(new_pos.1 as u32),
            0.1,
        );
        // update new player board position
        *mover_pos = BoardPosition {
            x: new_pos.0 as u32,
            y: new_pos.1 as u32,
        };
    }
}

fn simulation_ongoing(turn: Res<Turn>) -> bool {
    turn.num <= TURNS_PER_GEN
}
fn simulation_over_once(turn: Res<Turn>, its_joever: Res<GameOver>) -> bool {
    turn.num > TURNS_PER_GEN && !its_joever.0
}

fn player_eat_listener(
    mut commands: Commands,
    mut eat_events: EventReader<EatEvent>,
    board_state: Res<BoardState>,
    mut player_query: Query<(&BoardPosition, &mut Vitals), (With<Player>, Without<Food>)>,
    mut tile_query: Query<&mut OccupantType, With<BoardTile>>,
    food_query: Query<&Hunger, (With<Food>, Without<Player>)>,
) {
    for event in eat_events.read() {
        let (gorger_id, gorger_facing) = (event.gorger_id, event.gorger_facing);
        let maybe_query_res = player_query.get_mut(gorger_id);
        if maybe_query_res.is_err() {
            continue;
        }
        let (gorger_pos, mut gorger_vitals) = maybe_query_res.unwrap();
        let food_pos = predict_move_pos(gorger_pos, &gorger_facing);
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
                if let Ok(food_hunger) = food_query.get(food_id) {
                    gorger_vitals.hunger.value += food_hunger.value;
                    commands.entity(food_id).despawn_recursive();
                    *occ = OccupantType::Empty;
                }
            }
        }
    }
}

fn update_vitals_listener(
    mut uv_events: EventReader<UpdateVitalsEvent>,
    mut player_query: Query<
        (&mut Vitals, &mut Handle<ColorMaterial>),
        (With<Player>, Without<BoardTile>),
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for event in uv_events.read() {
        if let Ok((mut hungerer_vitals, mut hungerer_color)) =
            player_query.get_mut(event.hungerer_id)
        {
            hungerer_vitals.hunger.value -= 1;
            if hungerer_vitals.hunger.value == 0 {
                hungerer_vitals.status = PlayerStatus::DedPepega;
                *hungerer_color = materials.add(Color::rgb(0., 0., 0.));
            }
        }
    }
}

fn player_kill_listener(
    mut kill_event: EventReader<KillEvent>,
    mut commands: Commands,
    board_state: Res<BoardState>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut tile_query: Query<&mut OccupantType, With<BoardTile>>,
    mut player_query: Query<
        (
            &BoardPosition,
            &mut Vitals,
            &mut Handle<ColorMaterial>,
            &mut Mesh2dHandle,
        ),
        (With<Player>, Without<Food>),
    >,
) {
    for event in kill_event.read() {
        let (killer_id, killer_facing) = (event.killer_id, event.killer_facing);

        let maybe_query_res = player_query.get(killer_id);
        if maybe_query_res.is_err() {
            continue;
        }
        let (killer_pos, _, _, _) = maybe_query_res.unwrap();
        let victim_pos = predict_move_pos(killer_pos, &killer_facing);
        if !pos_within_bounds(&victim_pos) {
            continue;
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

        if let Ok((victim_pos, victim_vitals, _, _)) = player_query.get_mut(victim_id) {
            if victim_vitals.status == PlayerStatus::Alive {
                *victim_tile_occ = OccupantType::Empty;
                commands.entity(victim_id).despawn_recursive();
                place_food_at(
                    &mut commands,
                    *victim_pos,
                    FoodType::DeadMeat,
                    &board_state,
                    &mut tile_query,
                    &mut materials,
                    &mut meshes,
                )
                .unwrap();
            }
        }
    }
}

fn player_turn_listener(
    mut turn_events: EventReader<TurnEvent>,
    mut player_query: Query<
        (&mut FacingDirection, &mut Transform),
        (With<Player>, Without<BoardTile>),
    >,
) {
    for event in turn_events.read() {
        let maybe_query_res = player_query.get_mut(event.turner_id);
        if maybe_query_res.is_err() {
            continue;
        }
        if let Ok((mut turner_facing_mut, mut turner_transform)) = maybe_query_res {
            let turn_rad = match event.turn_direction {
                FacingDirection::Right => -PI / 2.,
                FacingDirection::Left => PI / 2.,
                _ => unreachable!("turn_event_listener(): can ONLY turn left or right!"),
            };

            *turner_facing_mut =
                position_after_turn(&event.turner_facing, event.turn_direction).unwrap();
            turner_transform.rotate_z(turn_rad);
        }
    }
}

fn advance_players(
    mut turn: ResMut<Turn>,
    mut kill_event: EventWriter<KillEvent>,
    mut eat_event: EventWriter<EatEvent>,
    mut move_event: EventWriter<MoveEvent>,
    mut turn_event: EventWriter<TurnEvent>,
    mut update_vitals_event: EventWriter<UpdateVitalsEvent>,
    mut player_query: Query<(Entity, &mut FacingDirection, &Vitals), With<Player>>,
) {
    for (player_id, direction, vitals) in player_query.iter_mut() {
        if vitals.status == PlayerStatus::DedPepega {
            continue;
        }
        match random_player_action() {
            PlayerActionType::Idle => (),
            PlayerActionType::Move => {
                move_event.send(MoveEvent {
                    mover_id: player_id,
                    mover_facing: *direction,
                });
            }
            PlayerActionType::Turn(FacingDirection::Right) => {
                turn_event.send(TurnEvent {
                    turner_id: player_id,
                    turner_facing: *direction,
                    turn_direction: FacingDirection::Right,
                });
            }
            PlayerActionType::Turn(FacingDirection::Left) => {
                turn_event.send(TurnEvent {
                    turner_id: player_id,
                    turner_facing: *direction,
                    turn_direction: FacingDirection::Left,
                });
            }
            PlayerActionType::Eat => {
                eat_event.send(EatEvent {
                    gorger_id: player_id,
                    gorger_facing: *direction,
                });
            }
            PlayerActionType::Kill => {
                kill_event.send(KillEvent {
                    killer_id: player_id,
                    killer_facing: *direction,
                });
            }
            act => unreachable!(
                "Incorrect action type while trying to advance players: {:#?}",
                act
            ),
        }
        update_vitals_event.send(UpdateVitalsEvent {
            hungerer_id: player_id,
        });
    }

    turn.num += 1;
}

fn log_survival_rate(player_query: Query<&Vitals, With<Player>>, mut its_joever: ResMut<GameOver>) {
    its_joever.0 = true;
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
        app.insert_resource(BoardState::new())
            .insert_resource(Time::<Fixed>::from_seconds(SECONDS_PER_TURN))
            .insert_resource(Turn::new())
            .insert_resource(GameOver(false))
            .add_event::<KillEvent>()
            .add_event::<EatEvent>()
            .add_event::<MoveEvent>()
            .add_event::<TurnEvent>()
            .add_event::<UpdateVitalsEvent>()
            .add_systems(Startup, (spawn_board, spawn_players, spawn_food).chain())
            .add_systems(FixedUpdate, advance_players.run_if(simulation_ongoing))
            .add_systems(
                Update,
                (
                    player_kill_listener,
                    player_eat_listener,
                    player_move_listener,
                    player_turn_listener,
                )
                    .run_if(simulation_ongoing)
                    .after(advance_players),
            )
            .add_systems(
                PostUpdate,
                update_vitals_listener.run_if(simulation_ongoing),
            )
            .add_systems(FixedLast, log_survival_rate.run_if(simulation_over_once));
    }
}

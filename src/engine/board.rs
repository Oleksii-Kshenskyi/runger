use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

use std::error::Error;
use std::f32::consts::PI;

use crate::{engine::common::*, engine::config::*, engine::random::*, simulation::players::*};

#[derive(Bundle)]
pub struct BoardTileBundle {
    pub pos: BoardPosition,
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
        Self { num: 0 }
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
    pub mover_pos: BoardPosition,
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

#[derive(Default, Clone, Copy, States, Debug, Hash, PartialEq, Eq)]
pub enum VisualizerState {
    #[default]
    SimulationRunning,
    GenerationFinished,
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
            board_state.add_tile(BoardPosition::new(x, y), tile_entity);
            board_state.add_occ(BoardPosition::new(x, y), OccupantType::Empty);
        }
    }
}

fn spawn_players(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut board_state: ResMut<BoardState>,
) {
    for _ in 0..default_player_count() {
        loop {
            let random_pos = BoardPosition::from_tuple(random_board_pos());

            if let Some(occupant) = board_state.occ_at_mut(&random_pos) {
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
    board_state: &mut ResMut<BoardState>,
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
    if let Some(occupant) = board_state.occ_at_mut(&pos) {
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

fn spawn_food(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut board_state: ResMut<BoardState>,
) {
    for _ in 0..default_food_count() {
        loop {
            let random_pos = BoardPosition::from_tuple(random_board_pos());
            if let Some(occ) = board_state.occ_at(&random_pos) {
                if *occ != OccupantType::Empty {
                    continue;
                }

                place_food_at(
                    &mut commands,
                    random_pos,
                    FoodType::Meal,
                    &mut board_state,
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
    mut player_query: Query<
        (&mut BoardPosition, &mut Transform),
        (With<Player>, Without<BoardTile>),
    >,
    mut board_state: ResMut<BoardState>,
) {
    for event in move_events.read() {
        let mut maybe_move_data: Option<(BoardPosition, OccupantType)> = None;
        if let Some((new_pos, new_tile_occ)) =
            board_state.looking_at(&event.mover_pos, &event.mover_facing)
        {
            if let Some(old_tile_occ) = board_state.occ_at(&event.mover_pos) {
                if *new_tile_occ == OccupantType::Empty {
                    // get data necessary for the move via immutable queries
                    maybe_move_data = Some((new_pos, *old_tile_occ));
                }
            }
        }

        if maybe_move_data.is_some() {
            if let Some(old_occ_mut) = board_state.occ_at_mut(&event.mover_pos) {
                *old_occ_mut = OccupantType::Empty; // deoccupy the old tile if the move is valid
            }
        }

        if let Some((new_pos, old_occ_clone)) = maybe_move_data {
            if let Ok((mut mover_pos, mut mover_transform)) = player_query.get_mut(event.mover_id) {
                if let Some((_, new_tile_occ)) =
                    board_state.looking_at_mut(&mover_pos, &event.mover_facing)
                {
                    // move player occupancy to the new position
                    *new_tile_occ = old_occ_clone;

                    // move player graphics (transform)
                    mover_transform.translation =
                        Vec3::new(grid_to_world(new_pos.x), grid_to_world(new_pos.y), 0.1);

                    // update new player board position
                    *mover_pos = BoardPosition {
                        x: new_pos.x,
                        y: new_pos.y,
                    };
                }
            }
        }
    }
}

fn player_eat_listener(
    mut commands: Commands,
    mut eat_events: EventReader<EatEvent>,
    mut board_state: ResMut<BoardState>,
    mut player_query: Query<(&BoardPosition, &mut Vitals), (With<Player>, Without<Food>)>,
    food_query: Query<&Hunger, (With<Food>, Without<Player>)>,
) {
    for event in eat_events.read() {
        if let Ok((gorger_pos, mut gorger_vitals)) = player_query.get_mut(event.gorger_id) {
            if let Some((_, occ)) = board_state.looking_at_mut(gorger_pos, &event.gorger_facing) {
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
    mut board_state: ResMut<BoardState>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
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
        if let Ok((killer_pos, _, _, _)) = player_query.get(event.killer_id) {
            if let Some((_, victim_tile_occ)) =
                board_state.looking_at_mut(killer_pos, &event.killer_facing)
            {
                if let OccupantType::Player(victim_id) = *victim_tile_occ {
                    if let Ok((victim_pos, victim_vitals, _, _)) = player_query.get_mut(victim_id) {
                        if victim_vitals.status == PlayerStatus::Alive {
                            *victim_tile_occ = OccupantType::Empty;
                            commands.entity(victim_id).despawn_recursive();
                            place_food_at(
                                &mut commands,
                                *victim_pos,
                                FoodType::DeadMeat,
                                &mut board_state,
                                &mut materials,
                                &mut meshes,
                            )
                            .unwrap();
                        }
                    }
                }
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
        if let Ok((mut turner_facing_mut, mut turner_transform)) =
            player_query.get_mut(event.turner_id)
        {
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

fn advance_turn(mut turn: ResMut<Turn>, mut states: ResMut<NextState<VisualizerState>>) {
    turn.num += 1;

    if turn.num > TURNS_PER_GEN {
        states.set(VisualizerState::GenerationFinished);
    }
}

fn advance_players(
    mut kill_event: EventWriter<KillEvent>,
    mut eat_event: EventWriter<EatEvent>,
    mut move_event: EventWriter<MoveEvent>,
    mut turn_event: EventWriter<TurnEvent>,
    mut update_vitals_event: EventWriter<UpdateVitalsEvent>,
    mut player_query: Query<
        (Entity, &BoardPosition, &mut FacingDirection, &Vitals),
        (With<Player>, Without<BoardTile>),
    >,
) {
    for (player_id, player_pos, direction, vitals) in player_query.iter_mut() {
        if vitals.status == PlayerStatus::DedPepega {
            continue;
        }
        match random_player_action() {
            PlayerActionType::Idle => (),
            PlayerActionType::Move => {
                move_event.send(MoveEvent {
                    mover_id: player_id,
                    mover_pos: *player_pos,
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
            .insert_resource(BoardState::new())
            .insert_resource(Time::<Fixed>::from_seconds(SECONDS_PER_TURN))
            .insert_resource(Turn::new())
            .add_event::<KillEvent>()
            .add_event::<EatEvent>()
            .add_event::<MoveEvent>()
            .add_event::<TurnEvent>()
            .add_event::<UpdateVitalsEvent>()
            .add_systems(Startup, (spawn_board, spawn_players, spawn_food).chain())
            .add_systems(
                FixedUpdate,
                (advance_players, advance_turn)
                    .run_if(in_state(VisualizerState::SimulationRunning)),
            )
            .add_systems(
                Update,
                (
                    player_turn_listener,
                    player_move_listener,
                    player_eat_listener,
                    player_kill_listener,
                )
                    .chain()
                    .run_if(in_state(VisualizerState::SimulationRunning))
                    .after(advance_players),
            )
            .add_systems(
                PostUpdate,
                update_vitals_listener.run_if(in_state(VisualizerState::SimulationRunning)),
            )
            .add_systems(
                OnEnter(VisualizerState::GenerationFinished),
                log_survival_rate,
            );
    }
}

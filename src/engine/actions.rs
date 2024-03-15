use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;

use crate::engine::common::*;
use crate::engine::random::random_player_action;
use crate::simulation::players::*;

use super::config::action_cost;

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

fn player_move_listener(
    mut move_events: EventReader<MoveEvent>,
    mut player_query: Query<
        (&mut BoardPosition, &mut PlayerActionType, &mut Transform),
        (With<Player>, Without<BoardTile>),
    >,
    mut board: ResMut<Board>,
) {
    for event in move_events.read() {
        let mut maybe_move_data: Option<(BoardPosition, OccupantType)> = None;
        if let Some((new_pos, new_tile_occ)) =
            board.looking_at(&event.mover_pos, &event.mover_facing)
        {
            if let Some(old_tile_occ) = board.occ_at(&event.mover_pos) {
                if *new_tile_occ == OccupantType::Empty {
                    // get data necessary for the move via immutable queries
                    maybe_move_data = Some((new_pos, *old_tile_occ));
                }
            }
        }

        if maybe_move_data.is_some() {
            if let Some(old_occ_mut) = board.occ_at_mut(&event.mover_pos) {
                *old_occ_mut = OccupantType::Empty; // deoccupy the old tile if the move is valid
            }
        }

        let mut move_succeeded = false;
        if let Some((new_pos, old_occ_clone)) = maybe_move_data {
            if let Ok((mut mover_pos, mut last_action, mut mover_transform)) = player_query.get_mut(event.mover_id) {
                if let Some((_, new_tile_occ)) =
                    board.looking_at_mut(&mover_pos, &event.mover_facing)
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

                    // log last action taken for action cost calculation
                    *last_action = PlayerActionType::Move;
                    move_succeeded = true;
                }
            }
        }
        if !move_succeeded {
            if let Ok((_, mut last_action, _)) = player_query.get_mut(event.mover_id) {
            *last_action = PlayerActionType::Idle;
            }
        }
    }
}

fn player_eat_listener(
    mut commands: Commands,
    mut eat_events: EventReader<EatEvent>,
    mut board: ResMut<Board>,
    mut player_query: Query<(&BoardPosition, &mut PlayerActionType, &mut Vitals), (With<Player>, Without<Food>)>,
    food_query: Query<&Energy, (With<Food>, Without<Player>)>,
) {
    for event in eat_events.read() {
        if let Ok((gorger_pos, mut last_action, mut gorger_vitals)) = player_query.get_mut(event.gorger_id) {
            *last_action = PlayerActionType::Idle;
            if let Some((_, occ)) = board.looking_at_mut(gorger_pos, &event.gorger_facing) {
                if let OccupantType::Food(food_id) = *occ {
                    if let Ok(food_energy) = food_query.get(food_id) {
                        gorger_vitals.energy.value += food_energy.value;
                        commands.entity(food_id).despawn_recursive();
                        *occ = OccupantType::Empty;

                        *last_action = PlayerActionType::Eat;
                    }
                }
            }
        }
    }
}

fn update_vitals_listener(
    mut uv_events: EventReader<UpdateVitalsEvent>,
    mut player_query: Query<
        (&mut Vitals, &mut Handle<ColorMaterial>, &PlayerActionType),
        (With<Player>, Without<BoardTile>),
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for event in uv_events.read() {
        if let Ok((mut hungerer_vitals, mut hungerer_color, last_action)) =
            player_query.get_mut(event.hungerer_id)
        {
            hungerer_vitals.energy.value = hungerer_vitals.energy.value.saturating_sub(action_cost(last_action));
            if hungerer_vitals.energy.value == 0 {
                hungerer_vitals.status = PlayerStatus::DedPepega;
                *hungerer_color = materials.add(Color::rgb(0., 0., 0.));
            }
        }
    }
}

fn player_kill_listener(
    mut kill_event: EventReader<KillEvent>,
    mut commands: Commands,
    mut board: ResMut<Board>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut player_query: Query<
        (
            &BoardPosition,
            &mut Vitals,
            &mut PlayerActionType,
            &mut Handle<ColorMaterial>,
            &mut Mesh2dHandle,
        ),
        (With<Player>, Without<Food>),
    >,
) {
    for event in kill_event.read() {
        let mut kill_succeeded = false;
        if let Ok((killer_pos, _, _, _, _)) = player_query.get(event.killer_id) {
            if let Some((_, victim_tile_occ)) =
                board.looking_at_mut(killer_pos, &event.killer_facing)
            {
                if let OccupantType::Player(victim_id) = *victim_tile_occ {
                    if let Ok((victim_pos, victim_vitals, _, _, _)) = player_query.get_mut(victim_id) {
                        if victim_vitals.status == PlayerStatus::Alive {
                            *victim_tile_occ = OccupantType::Empty;
                            commands.entity(victim_id).despawn_recursive();
                            place_food_at(
                                &mut commands,
                                *victim_pos,
                                FoodType::DeadMeat,
                                &mut board,
                                &mut materials,
                                &mut meshes,
                            )
                            .unwrap();
                            kill_succeeded = true;
                        }
                    }
                }
            }
        }
        if let Ok((_, _, mut last_killer_action, _, _)) = player_query.get_mut(event.killer_id) {
            *last_killer_action = match kill_succeeded {
                true => PlayerActionType::Kill,
                false => PlayerActionType::Idle,
            }
        }
    }
}

fn player_turn_listener(
    mut turn_events: EventReader<TurnEvent>,
    mut player_query: Query<
        (&mut FacingDirection, &mut PlayerActionType, &mut Transform),
        (With<Player>, Without<BoardTile>),
    >,
) {
    for event in turn_events.read() {
        if let Ok((mut turner_facing_mut, mut last_action, mut turner_transform)) =
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
            *last_action = PlayerActionType::Turn(event.turn_direction);
        }
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

pub struct PlayerActionPlugin;

impl Plugin for PlayerActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<KillEvent>()
            .add_event::<EatEvent>()
            .add_event::<MoveEvent>()
            .add_event::<TurnEvent>()
            .add_event::<UpdateVitalsEvent>()
            .add_systems(
                FixedUpdate,
                (advance_players).run_if(in_state(VisualizerState::SimulationRunning)),
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
            );
    }
}

use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;

use crate::engine::common::*;
use crate::engine::random::random_player_action;
use crate::simulation::players::*;

use super::config::{
    action_cost, default_entity_size, DEFAULT_COLOR_ON_LOS_DETECT, DEFAULT_WALL_COLOR,
};

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
    pub movement_direction: FacingDirection,
}

#[derive(Event, Debug)]
pub struct TurnEvent {
    pub turner_id: Entity,
    pub turner_facing: FacingDirection,
    pub turn_direction: FacingDirection,
}

#[derive(Event, Debug)]
pub struct BuildWallEvent {
    pub builder_id: Entity,
}

#[derive(Event, Debug)]
pub struct ScanLOSEvent {
    pub scanner_id: Entity,
    pub scanner_facing: FacingDirection,
}

#[derive(Event, Debug)]
pub struct LOSReportEvent {
    pub scanner_id: Entity,
    pub scanned_type: OccupantType,
}

#[derive(Event, Debug)]
pub struct RestoreColorsEvent {
    pub entity_id: Entity,
    pub old_color: Color,
}

#[derive(Event, Debug)]
pub struct UpdateVitalsEvent {
    pub hungerer_id: Entity,
}

fn move_player(
    board: &mut Board,
    mover_id: Entity,
    mover_pos: &BoardPosition,
    mover_facing: &FacingDirection,
    movement_direction: &FacingDirection,
    player_query: &mut Query<
        (&mut BoardPosition, &mut PlayerActionType, &mut Transform),
        (With<Player>, Without<BoardTile>),
    >,
) -> bool {
    let movement_fn = match movement_direction {
        FacingDirection::Up => Board::looking_at,
        FacingDirection::Down => Board::disengage_to,
        _ => unreachable!("ERROR: ILLEGAL MOVE: can only move forwards or backwards."),
    };
    let movement_fn_mut = match movement_direction {
        FacingDirection::Up => Board::looking_at_mut,
        FacingDirection::Down => Board::disengage_to_mut,
        _ => unreachable!("ERROR: ILLEGAL MOVE (MUT): can only move forwards or backwards."),
    };

    let mut maybe_move_data: Option<(BoardPosition, OccupantType)> = None;
    if let Some((new_pos, new_tile_occ)) = movement_fn(board, mover_pos, mover_facing, None) {
        if let Some(old_tile_occ) = board.occ_at(mover_pos) {
            if *new_tile_occ == OccupantType::Empty {
                // get data necessary for the move via immutable queries
                maybe_move_data = Some((new_pos, *old_tile_occ));
            }
        }
    }

    if maybe_move_data.is_some() {
        if let Some(old_occ_mut) = board.occ_at_mut(mover_pos) {
            *old_occ_mut = OccupantType::Empty; // deoccupy the old tile if the move is valid
        }
    }

    let mut move_succeeded = false;
    if let Some((new_pos, old_occ_clone)) = maybe_move_data {
        if let Ok((mut mover_pos, mut last_action, mut mover_transform)) =
            player_query.get_mut(mover_id)
        {
            if let Some((_, new_tile_occ)) = movement_fn_mut(board, &mover_pos, mover_facing, None) {
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
                *last_action = PlayerActionType::MoveForward;
                move_succeeded = true;
            }
        }
    }

    move_succeeded
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
        if event.movement_direction != FacingDirection::Up {
            continue;
        }
        let ok = move_player(
            &mut board,
            event.mover_id,
            &event.mover_pos,
            &event.mover_facing,
            &FacingDirection::Up,
            &mut player_query,
        );
        if let Ok((_, mut last_action, _)) = player_query.get_mut(event.mover_id) {
            match ok {
                true => *last_action = PlayerActionType::MoveForward,
                false => *last_action = PlayerActionType::Idle,
            }
        }
    }
}

fn player_eat_listener(
    mut commands: Commands,
    mut eat_events: EventReader<EatEvent>,
    mut board: ResMut<Board>,
    mut player_query: Query<
        (&BoardPosition, &mut PlayerActionType, &mut Vitals),
        (With<Player>, Without<Food>),
    >,
    food_query: Query<&Energy, (With<Food>, Without<Player>)>,
) {
    for event in eat_events.read() {
        if let Ok((gorger_pos, mut last_action, mut gorger_vitals)) =
            player_query.get_mut(event.gorger_id)
        {
            *last_action = PlayerActionType::Idle;
            if let Some((_, occ)) = board.looking_at_mut(gorger_pos, &event.gorger_facing, None) {
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
            hungerer_vitals.energy.value = hungerer_vitals
                .energy
                .value
                .saturating_sub(action_cost(last_action));
            if hungerer_vitals.energy.value == 0 {
                hungerer_vitals.status = PlayerStatus::DedPepega;
                *hungerer_color = materials.add(Color::srgb(0., 0., 0.));
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
                board.looking_at_mut(killer_pos, &event.killer_facing, None)
            {
                if let OccupantType::Player(victim_id) = *victim_tile_occ {
                    if let Ok((victim_pos, victim_vitals, _, _, _)) =
                        player_query.get_mut(victim_id)
                    {
                        if victim_vitals.status == PlayerStatus::Alive {
                            *victim_tile_occ = OccupantType::Empty;
                            commands.entity(victim_id).despawn_recursive();
                            if let Err(e) = place_food_at(
                                &mut commands,
                                *victim_pos,
                                FoodType::DeadMeat(victim_vitals.energy.value),
                                &mut board,
                                &mut materials,
                                &mut meshes,
                            ) {
                                warn!("Tried to place a dead body, but failed: `{}`", e);
                            }
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

fn player_build_wall_listener(
    mut build_wall_events: EventReader<BuildWallEvent>,
    mut commands: Commands,
    mut board: ResMut<Board>,
    mut player_query: Query<
        (&FacingDirection, &mut PlayerActionType, &BoardPosition),
        (With<Player>, Without<BoardTile>),
    >,
) {
    for event in build_wall_events.read() {
        let mut maybe_empty: Option<BoardPosition> = None;
        let mut last_action_type: PlayerActionType = PlayerActionType::Idle;
        if let Ok((builder_facing, _, builder_pos)) = player_query.get(event.builder_id) {
            if let Some((occ_pos, occ_type)) = board.looking_at(builder_pos, builder_facing, None) {
                maybe_empty = if *occ_type == OccupantType::Empty {
                    Some(occ_pos)
                } else {
                    None
                }
            }
        }
        if let Some(wall_pos) = maybe_empty {
            if let Some(occ) = board.occ_at_mut(&wall_pos) {
                *occ = OccupantType::Wall(
                    commands
                        .spawn((
                            WallBundle {
                                board_pos: wall_pos,
                                sprite: SpriteBundle {
                                    sprite: Sprite {
                                        color: DEFAULT_WALL_COLOR,
                                        custom_size: Some(Vec2::new(
                                            default_entity_size(),
                                            default_entity_size(),
                                        )),
                                        ..default()
                                    },
                                    transform: Transform::from_xyz(
                                        grid_to_world(wall_pos.x),
                                        grid_to_world(wall_pos.y),
                                        0.1,
                                    ),
                                    ..Default::default()
                                },
                            },
                            Wall,
                        ))
                        .id(),
                );
                last_action_type = PlayerActionType::BuildWall;
            }
        } else {
            last_action_type = PlayerActionType::Idle;
        }

        if let Ok((_, mut last_action, _)) = player_query.get_mut(event.builder_id) {
            *last_action = last_action_type;
        }
    }
}

/// What should this system do?
/// It should scan line of sight of the current player and determine if
/// there is something within the line of sight.
/// For now, we're doing simple straight line directly in front of the player,
/// but in the future there should be more options for different types of LOS, like circle, cone etc.
/// If there IS something within the player's line of sight, report it via firing a different event. All the systems that need LOS are going to respond with subscribing to those.
/// In the new event, we need to basically report OccupantType and BoardPosition of what we're seeing (and don't forget the scanner's Entity ID itself).
fn player_scan_los_listener(
    mut scanlos_events: EventReader<ScanLOSEvent>,
    mut losreport_events: EventWriter<LOSReportEvent>,
    board: Res<Board>,
    mut player_query: Query<(&BoardPosition, &mut PlayerActionType, &LineOfSight), With<Player>>,
) {
    for event in scanlos_events.read() {
        let maybe_scanner = if let Ok((pos, _, los)) = player_query.get(event.scanner_id) {
            Some((*pos, *los))
        } else {
            None
        };
        if let Some((pos, los)) = maybe_scanner {
            let tiles_to_scan = get_los_tiles(&pos, &event.scanner_facing, &los, &board);

            for pos in tiles_to_scan {
                if let Some(occ) = board.occ_at(&pos) {
                    if *occ != OccupantType::Empty {
                        losreport_events.send(LOSReportEvent {
                            scanned_type: *occ,
                            scanner_id: event.scanner_id,
                        });
                        break;
                    }
                }
            }
        }

        if let Ok((_, mut last_action, _)) = player_query.get_mut(event.scanner_id) {
            *last_action = PlayerActionType::ScanLOS;
        }
    }
}

fn player_los_report_listener(
    mut los_report_events: EventReader<LOSReportEvent>,
    mut restore_colors_event: EventWriter<RestoreColorsEvent>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut color_query: Query<&mut Handle<ColorMaterial>>,
) {
    for event in los_report_events.read() {
        if let Ok(mut scanner_color) = color_query.get_mut(event.scanner_id) {
            let current_color = materials.get(scanner_color.as_ref()).unwrap().color;
            if current_color != DEFAULT_COLOR_ON_LOS_DETECT {
                restore_colors_event.send(RestoreColorsEvent {
                    entity_id: event.scanner_id,
                    old_color: current_color,
                });
                *scanner_color = materials.add(DEFAULT_COLOR_ON_LOS_DETECT);
            }
        }
        let scanned_id = match event.scanned_type {
            OccupantType::Player(player_id) => player_id,
            OccupantType::Food(food_id) => food_id,
            OccupantType::Wall(wall_id) => wall_id,
            err_occ => unreachable!(
                "Changing color on LOS: this entity type should not be scanned: {:?}",
                err_occ
            ),
        };
        if let Ok(mut scanned_color) = color_query.get_mut(scanned_id) {
            let current_color = materials.get(scanned_color.as_ref()).unwrap().color;
            if current_color != DEFAULT_COLOR_ON_LOS_DETECT {
                restore_colors_event.send(RestoreColorsEvent {
                    entity_id: scanned_id,
                    old_color: current_color,
                });
                *scanned_color = materials.add(DEFAULT_COLOR_ON_LOS_DETECT);
            }
        }
    }
}

fn restore_colors_listener(
    mut restore_colors_events: EventReader<RestoreColorsEvent>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut color_query: Query<&mut Handle<ColorMaterial>>,
) {
    for event in restore_colors_events.read() {
        if let Ok(mut color) = color_query.get_mut(event.entity_id) {
            *color = materials.add(event.old_color);
        }
    }
}

fn advance_players(
    mut kill_event: EventWriter<KillEvent>,
    mut eat_event: EventWriter<EatEvent>,
    mut move_event: EventWriter<MoveEvent>,
    mut turn_event: EventWriter<TurnEvent>,
    mut los_event: EventWriter<ScanLOSEvent>,
    mut build_wall_event: EventWriter<BuildWallEvent>,
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
            PlayerActionType::MoveForward => {
                move_event.send(MoveEvent {
                    mover_id: player_id,
                    mover_pos: *player_pos,
                    mover_facing: *direction,
                    movement_direction: FacingDirection::Up,
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
            PlayerActionType::ScanLOS => {
                los_event.send(ScanLOSEvent {
                    scanner_id: player_id,
                    scanner_facing: *direction,
                });
            }
            PlayerActionType::MoveBackwards => {
                move_event.send(MoveEvent {
                    mover_id: player_id,
                    mover_facing: *direction,
                    mover_pos: *player_pos,
                    movement_direction: FacingDirection::Down,
                });
            }
            PlayerActionType::BuildWall => {
                build_wall_event.send(BuildWallEvent {
                    builder_id: player_id,
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
            .add_event::<BuildWallEvent>()
            .add_event::<ScanLOSEvent>()
            .add_event::<LOSReportEvent>()
            .add_event::<UpdateVitalsEvent>()
            .add_event::<RestoreColorsEvent>()
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
                    player_build_wall_listener,
                    player_scan_los_listener,
                    player_los_report_listener,
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
                FixedPostUpdate,
                restore_colors_listener.after(player_los_report_listener),
            );
    }
}

use bevy::prelude::*;

// TODO: Introduce the concept of types of player action:
// -- The player can choose from certain types of action (move or turn for now)
// -- Each turn, the player decides (either randomly or based on its "brain") which action from the types of actions available to take
#[derive(Component, Debug)]
pub struct Player;

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
mod engine;
mod simulation;

use bevy::prelude::*;

use engine::actions::PlayerActionPlugin;
use engine::board::GameBoardPlugin;
use engine::rsystem::BaseSystemPlugin;

fn main() {
    App::new()
        .add_plugins(BaseSystemPlugin)
        .add_plugins(GameBoardPlugin)
        .add_plugins(PlayerActionPlugin)
        .run();
}

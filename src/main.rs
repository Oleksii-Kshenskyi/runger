mod engine;

use bevy::prelude::*;

use engine::board::GameBoardPlugin;
use engine::rsystem::BaseSystemPlugin;

fn main() {
    App::new()
        .add_plugins(BaseSystemPlugin)
        .add_plugins(GameBoardPlugin)
        .run();
}

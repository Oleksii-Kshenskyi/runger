use bevy::{prelude::*, render::pass::ClearColor};

fn main() {
    App::build()
        .add_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .add_plugins(DefaultPlugins)
        .run();
}
use bevy::prelude::*;
use bevy::window::WindowResolution;

fn camera_setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

pub struct BaseSystemPlugin;

impl Plugin for BaseSystemPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::rgb_u8(25, 25, 25)))
            .add_systems(Startup, camera_setup)
            .add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Runger: The Hunger Games simulation".to_string(),
                    resolution: WindowResolution::new(1280., 720.).with_scale_factor_override(1.0),
                    ..default()
                }),
                ..default()
            }));
    }
}

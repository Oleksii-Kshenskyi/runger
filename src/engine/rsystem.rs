use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};

fn camera_setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

pub struct BaseSystemPlugin;

impl Plugin for BaseSystemPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb_u8(25, 25, 25)))
            .add_systems(Startup, camera_setup)
            .add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Runger: The Hunger Games simulation".to_string(),
                    resolution: WindowResolution::new(1920., 1080.).with_scale_factor_override(1.0),
                    position: WindowPosition::Centered(MonitorSelection::Primary),
                    present_mode: PresentMode::Fifo,
                    ..default()
                }),
                ..default()
            }));
    }
}

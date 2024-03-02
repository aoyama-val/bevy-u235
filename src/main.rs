use bevy::{
    prelude::*,
    window::{PresentMode, WindowTheme},
};

const TITLE: &str = "u235";
const SCREEEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 400.0;
const CELL_SIZE_PX: f32 = 16.0;
const FPS: f32 = 30.0;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: TITLE.into(),
                    name: Some(TITLE.into()),
                    resolution: (SCREEEN_WIDTH, SCREEN_HEIGHT).into(),
                    present_mode: PresentMode::AutoVsync,
                    // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                    // prevent_default_event_handling: false,
                    // window_theme: Some(WindowTheme::Dark),
                    // enabled_buttons: bevy::window::EnabledButtons {
                    //     maximize: false,
                    //     ..Default::default()
                    // },
                    // This will spawn an invisible window
                    // The window will be made visible in the make_visible() system after 3 frames.
                    // This is useful when you want to avoid the white window that shows up before the GPU is ready to render the app.
                    // visible: false,
                    ..default()
                }),
                ..default()
            }),
            // LogDiagnosticsPlugin::default(),
            // FrameTimeDiagnosticsPlugin,
        ))
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_systems(Startup, setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(SCREEEN_WIDTH / 2.0, -SCREEN_HEIGHT / 2.0, 999.0),
        projection: OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::WindowSize(1.0),
            ..default()
        },
        // rotation: Quat::from_rotation_z(-45.0_f32.to_radians()),
        ..default()
    });
    commands.spawn(SpriteBundle {
        texture: asset_server.load("image/player.png"),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });
    commands.spawn(SpriteBundle {
        texture: asset_server.load("image/target.png"),
        transform: Transform::from_xyz(SCREEEN_WIDTH, -SCREEN_HEIGHT, 0.0),
        ..default()
    });
}

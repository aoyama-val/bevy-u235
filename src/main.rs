use bevy::{
    prelude::*,
    transform,
    window::{PresentMode, WindowTheme},
};

const TITLE: &str = "u235";
const SCREEEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 400.0;
const CELL_SIZE_PX: f32 = 16.0;
const FPS: f32 = 30.0;
const X_MIN: i32 = 2;
const X_MAX: i32 = 640 / 16 - 3;
const Y_MIN: i32 = 2;
const Y_MAX: i32 = 400 / 16 - 3;

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
    let mut transform = Transform::from_xyz(SCREEEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0, 999.0);
    // transform.rotate(Quat::from_euler(
    //     EulerRot::XYZ,
    //     180.0f32.to_radians(),
    //     0.0,
    //     0.0,
    // ));
    // transform.with_translation (SCREEEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0, -999.0);
    commands.spawn(Camera2dBundle {
        transform: transform,
        projection: OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::WindowSize(1.0),
            ..default()
        },
        // rotation: Quat::from_rotation_z(-45.0_f32.to_radians()),
        ..default()
    });

    let sprite = Sprite {
        anchor: bevy::sprite::Anchor::TopLeft,
        ..Default::default()
    };

    // render player
    let player_texture = asset_server.load("image/player.png");
    commands.spawn(SpriteBundle {
        texture: player_texture,
        transform: Transform::from_xyz(
            // 0.0,
            // SCREEN_HEIGHT,
            CELL_SIZE_PX * 18.0,
            SCREEN_HEIGHT - CELL_SIZE_PX * Y_MAX as f32,
            0.0,
        ),
        sprite: sprite.clone(),
        ..default()
    });
    // commands.spawn(SpriteBundle {
    //     texture: asset_server.load("image/target.png"),
    //     transform: Transform::from_xyz(SCREEEN_WIDTH, SCREEN_HEIGHT / 2.0, 0.0),
    //     ..default()
    // });

    // render wall
    for y in 1..=Y_MAX {
        commands.spawn(SpriteBundle {
            texture: asset_server.load("image/wall.png"),
            transform: cell_to_transform(1, y),
            sprite: sprite.clone(),
            ..default()
        });
        commands.spawn(SpriteBundle {
            texture: asset_server.load("image/wall.png"),
            transform: cell_to_transform(X_MAX + 1, y),
            sprite: sprite.clone(),
            ..default()
        });
    }
    for x in 1..(X_MAX + 1) {
        commands.spawn(SpriteBundle {
            texture: asset_server.load("image/wall.png"),
            transform: cell_to_transform(x, 1),
            sprite: sprite.clone(),
            ..default()
        });
    }

    // render back
    for i in (X_MIN - 2)..=(X_MAX + 2) {
        commands.spawn(SpriteBundle {
            texture: asset_server.load("image/back.png"),
            transform: cell_to_transform(i, Y_MAX + 1),
            sprite: sprite.clone(),
            ..default()
        });
    }

    // render title
    commands.spawn(SpriteBundle {
        texture: asset_server.load("image/title.png"),
        transform: cell_to_transform(1, 0),
        sprite: sprite.clone(),
        ..default()
    });
}

fn cell_to_transform(cx: i32, cy: i32) -> Transform {
    Transform::from_xyz(
        CELL_SIZE_PX * cx as f32,
        SCREEN_HEIGHT - CELL_SIZE_PX * cy as f32,
        0.0,
    )
}

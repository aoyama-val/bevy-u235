use bevy::{
    prelude::*,
    transform,
    window::{PresentMode, WindowTheme},
};
use bevy_framepace::Limiter;

const TITLE: &str = "u235";
const SCREEEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 400.0;
const CELL_SIZE_PX: f32 = 16.0;
const FPS: f32 = 30.0;
const X_MIN: i32 = 2;
const X_MAX: i32 = 640 / 16 - 3;
const Y_MIN: i32 = 2;
const Y_MAX: i32 = 400 / 16 - 3;

// #[derive(Component)]
// struct X(i32);
// #[derive(Component)]
// struct Y(i32);

#[derive(Debug, Clone, PartialEq, Eq, Component)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Component)]
struct Player;

// #[derive(PartialEq, Eq)]
// enum MoveEvent {
//     Left,
//     Right,
// }

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let mut player_transform = query.single_mut();
    // let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        player_transform.translation.x = (player_transform.translation.x - CELL_SIZE_PX);
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        player_transform.translation.x = (player_transform.translation.x + CELL_SIZE_PX);
    }
    player_transform.translation.x = player_transform.translation.x.clamp(
        CELL_SIZE_PX * X_MIN as f32,
        CELL_SIZE_PX * (X_MAX - 2) as f32,
    );

    if keyboard_input.just_pressed(KeyCode::Space) {
        println!("Space just pressed");
    }

    // // Calculate the new horizontal paddle position based on player input
    // let new_paddle_position =
    //     paddle_transform.translation.x + direction * PADDLE_SPEED * time.delta_seconds();

    // // Update the paddle position,
    // // making sure it doesn't cause the paddle to leave the arena
    // let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
    // let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;
}

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
            bevy::diagnostic::LogDiagnosticsPlugin::default(),
            bevy_framepace::FramepacePlugin,
            bevy_framepace::debug::DiagnosticsPlugin,
            // LogDiagnosticsPlugin::default(),
            // FrameTimeDiagnosticsPlugin,
        ))
        .insert_resource(bevy_framepace::FramepaceSettings {
            limiter: Limiter::from_framerate(30.0),
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                move_player,
                // check_for_collisions,
                // play_collision_sound,
            )
                .chain(),
        )
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(SCREEEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0, 999.0),
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

    // Player
    let player_position = Position::new(18, Y_MAX);
    let player_texture = asset_server.load("image/player.png");
    commands.spawn((
        SpriteBundle {
            texture: player_texture,
            transform: position_to_transform(Position::new(18, Y_MAX)),
            sprite: sprite.clone(),
            ..default()
        },
        player_position,
        Player,
    ));

    // Walls
    for y in 1..=Y_MAX {
        commands.spawn(SpriteBundle {
            texture: asset_server.load("image/wall.png"),
            transform: position_to_transform(Position::new(1, y)),
            sprite: sprite.clone(),
            ..default()
        });
        commands.spawn(SpriteBundle {
            texture: asset_server.load("image/wall.png"),
            transform: position_to_transform(Position::new(X_MAX + 1, y)),
            sprite: sprite.clone(),
            ..default()
        });
    }
    for x in 1..(X_MAX + 1) {
        commands.spawn(SpriteBundle {
            texture: asset_server.load("image/wall.png"),
            transform: position_to_transform(Position::new(x, 1)),
            sprite: sprite.clone(),
            ..default()
        });
    }

    // Back
    for i in (X_MIN - 2)..=(X_MAX + 2) {
        commands.spawn(SpriteBundle {
            texture: asset_server.load("image/back.png"),
            transform: position_to_transform(Position::new(i, Y_MAX + 1)),
            sprite: sprite.clone(),
            ..default()
        });
    }

    // Title
    commands.spawn(SpriteBundle {
        texture: asset_server.load("image/title.png"),
        transform: position_to_transform(Position::new(1, 0)),
        sprite: sprite.clone(),
        ..default()
    });

    // // Sound
    // let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    // commands.insert_resource(CollisionSound(ball_collision_sound));
}

fn position_to_transform(position: Position) -> Transform {
    Transform::from_xyz(
        CELL_SIZE_PX * position.x as f32,
        SCREEN_HEIGHT - CELL_SIZE_PX * position.y as f32,
        0.0,
    )
}

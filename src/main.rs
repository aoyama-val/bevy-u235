use bevy::{prelude::*, window::PresentMode};
use bevy_framepace::Limiter;

const TITLE: &str = "u235";
const SCREEEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 400.0;
const CELL_SIZE_PX: f32 = 16.0;
const FPS: f64 = 30.0;
const X_MIN: i32 = 2;
const X_MAX: i32 = 640 / 16 - 3;
const Y_MIN: i32 = 2;
const Y_MAX: i32 = 400 / 16 - 3;

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

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: TITLE.into(),
                    name: Some(TITLE.into()),
                    resolution: (SCREEEN_WIDTH, SCREEN_HEIGHT).into(),
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            }),
            bevy_framepace::FramepacePlugin,
        ))
        .insert_resource(bevy_framepace::FramepaceSettings {
            limiter: Limiter::from_framerate(FPS),
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_systems(Startup, setup)
        .add_systems(Update, (move_player, bevy::window::close_on_esc))
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

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let mut player_transform = query.single_mut();

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        // println!("{}: left pressed", time.elapsed_seconds());
        player_transform.translation.x = (player_transform.translation.x - CELL_SIZE_PX);
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        // println!("{}: right pressed", time.elapsed_seconds());
        player_transform.translation.x = (player_transform.translation.x + CELL_SIZE_PX);
    }
    player_transform.translation.x = player_transform.translation.x.clamp(
        CELL_SIZE_PX * X_MIN as f32,
        CELL_SIZE_PX * (X_MAX - 2) as f32,
    );

    if keyboard_input.just_pressed(KeyCode::Space) {
        println!("Space just pressed");
    }
}

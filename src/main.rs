use bevy::{
    prelude::*,
    window::{Cursor, PresentMode},
};
use bevy_framepace::Limiter;

const TITLE: &str = "u235";
const SCREEEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 400.0;
const CELL_SIZE_PX: f32 = 16.0;
const FPS: f64 = 30.0;
const X_MIN: i32 = 2;
const X_MAX: i32 = 640 / 16 - 3;
// const Y_MIN: i32 = 2;
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

#[derive(Resource)]
struct HitSound(Handle<AudioSource>);

#[derive(Event, Default)]
struct ShootEvent;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: TITLE.into(),
                    name: Some(TITLE.into()),
                    resolution: (SCREEEN_WIDTH, SCREEN_HEIGHT).into(),
                    present_mode: PresentMode::AutoVsync,
                    cursor: Cursor {
                        visible: false,
                        ..default()
                    },
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
        .add_event::<ShootEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (move_player, bevy::window::close_on_esc, play_shoot_sound).chain(),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    // ワールド座標は画面左上を(0, 400)、右下を(640, 0)とする
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

    // Sound
    let hit_sound = asset_server.load("sound/hit.wav");
    commands.insert_resource(HitSound(hit_sound));
}

// セル座標をワールド座標に変換する
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
    mut shoot_events: EventWriter<ShootEvent>,
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
        shoot_events.send_default();
    }
}

fn play_shoot_sound(
    mut commands: Commands,
    mut shoot_events: EventReader<ShootEvent>,
    sound: Res<HitSound>,
) {
    // Play a sound once per frame if a collision occurred.
    if !shoot_events.is_empty() {
        // This prevents events staying active on the next frame.
        shoot_events.clear();
        commands.spawn(AudioBundle {
            source: sound.0.clone(),
            // auto-despawn the entity when playback finishes
            settings: PlaybackSettings::DESPAWN,
        });
    }
}

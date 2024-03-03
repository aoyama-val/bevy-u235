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

#[derive(Debug, Default, Clone, PartialEq, Eq, Component)]
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

#[derive(Component)]
struct Bullet;

#[derive(Resource, Default)]
struct Game {
    score: i32,
    hi_score: i32,
    bullet_handles: [Handle<Image>; 4],
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
        .init_resource::<Game>()
        .insert_resource(bevy_framepace::FramepaceSettings {
            limiter: Limiter::from_framerate(FPS),
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_event::<ShootEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_player,
                move_bullets,
                bevy::window::close_on_esc,
                play_shoot_sound,
            )
                .chain(),
        )
        .run();
}

fn create_default_sprite() -> Sprite {
    Sprite {
        anchor: bevy::sprite::Anchor::TopLeft,
        ..Default::default()
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut game: ResMut<Game>) {
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

    let sprite: Sprite = create_default_sprite();

    // Player
    let player_position = Position::new(18, Y_MAX);
    commands.spawn((
        Player,
        player_position.clone(),
        SpriteBundle {
            texture: asset_server.load("image/player.png"),
            transform: position_to_transform(player_position.clone()),
            sprite: sprite.clone(),
            ..default()
        },
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

    game.bullet_handles[0] = asset_server.load("image/left.png");
    game.bullet_handles[1] = asset_server.load("image/right.png");
    game.bullet_handles[2] = asset_server.load("image/up.png");
    game.bullet_handles[3] = asset_server.load("image/down.png");

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
    mut position_query: Query<&mut Position, With<Player>>,
    time: Res<Time>,
    mut shoot_events: EventWriter<ShootEvent>,
    mut commands: Commands,
    mut game: ResMut<Game>,
) {
    let mut player_transform = query.single_mut();
    let mut player_position = position_query.single_mut();

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        if player_position.x > X_MIN {
            player_position.x = player_position.x - 1;
        }
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        if player_position.x < X_MAX - 2 {
            player_position.x = player_position.x + 1;
        }
    }
    player_transform.translation = position_to_transform(player_position.clone()).translation;

    if keyboard_input.pressed(KeyCode::Space) {
        // 発射時は音を出さないのだった
        // shoot_events.send_default();

        let bullet_position = Position::new(player_position.x + 1, player_position.y - 1);
        spawn_bullet(commands, game, &bullet_position);
    }
}

fn spawn_bullet(mut commands: Commands, mut game: ResMut<Game>, bullet_position: &Position) {
    commands.spawn((
        Bullet,
        bullet_position.clone(),
        SpriteBundle {
            texture: game.bullet_handles[2].clone(),
            transform: position_to_transform(bullet_position.clone()),
            sprite: create_default_sprite(),
            ..default()
        },
    ));
}

fn move_bullets(mut query: Query<(&mut Position, &mut Transform), With<Bullet>>) {
    for (mut pos, mut transform) in &mut query {
        pos.y -= 1;
        *transform = position_to_transform(pos.clone());
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

use bevy::{
    prelude::*,
    window::{Cursor, PresentMode},
};
use bevy_framepace::Limiter;
use rand::Rng;

const TITLE: &str = "u235";
const SCREEEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 400.0;
const CELL_SIZE_PX: f32 = 16.0;
const FPS: f64 = 30.0;
const X_MIN: i32 = 2;
const X_MAX: i32 = 640 / 16 - 3;
const Y_MIN: i32 = 2;
const Y_MAX: i32 = 400 / 16 - 3;

const DIRECTION_LEFT: usize = 0;
const DIRECTION_RIGHT: usize = 1;
const DIRECTION_UP: usize = 2;
const DIRECTION_DOWN: usize = 3;

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

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct Target;

#[derive(Component)]
struct Direction(usize);

#[derive(Component)]
struct NumberType(&'static str, usize);

#[derive(Resource)]
struct HitSound(Handle<AudioSource>);

#[derive(Resource, Default)]
struct Game {
    score: i32,
    hi_score: i32,
    bullet_handles: [Handle<Image>; 4],
    texture_atlas_layout: Handle<TextureAtlasLayout>,
    number_texture: Handle<Image>,
}

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
        .init_resource::<Game>()
        .insert_resource(bevy_framepace::FramepaceSettings {
            limiter: Limiter::from_framerate(FPS),
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_event::<ShootEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_player,
                move_bullets,
                spawn_target,
                check_for_bullet_target_collisions,
                update_score,
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

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game: ResMut<Game>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
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

    let texture: Handle<Image> = asset_server.load("image/numbers.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(8.0, 16.0), 10, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    game.texture_atlas_layout = texture_atlas_layout;
    game.number_texture = texture;

    // Score, HiScore
    game.score = 0;
    game.hi_score = 0;
    spawn_number(game.hi_score, 18, 0, &mut commands, &mut game, "HiScore");
    spawn_number(game.score, 32, 0, &mut commands, &mut game, "Score");

    game.bullet_handles[DIRECTION_LEFT] = asset_server.load("image/left.png");
    game.bullet_handles[DIRECTION_RIGHT] = asset_server.load("image/right.png");
    game.bullet_handles[DIRECTION_UP] = asset_server.load("image/up.png");
    game.bullet_handles[DIRECTION_DOWN] = asset_server.load("image/down.png");

    // Sound
    let hit_sound = asset_server.load("sound/hit.wav");
    commands.insert_resource(HitSound(hit_sound));
}

fn spawn_number(
    num: i32,
    cx: i32,
    cy: i32,
    commands: &mut Commands,
    game: &ResMut<Game>,
    label: &'static str,
) {
    let text = format!("{:8}", num);
    let mut numbers_pos = position_to_transform(Position::new(cx, cy));
    for i in 0..8 {
        let byte = text.as_bytes()[i];
        if 0x30 <= byte && byte <= 0x39 {
            let index = (byte - 0x30) as usize;
            commands.spawn((
                SpriteSheetBundle {
                    texture: game.number_texture.clone(),
                    atlas: TextureAtlas {
                        layout: game.texture_atlas_layout.clone(),
                        index: index,
                    },
                    transform: numbers_pos,
                    visibility: Visibility::Visible,
                    sprite: create_default_sprite(),
                    ..default()
                },
                NumberType(label, i),
            ));
        }
        numbers_pos.translation.x += 8.0;
    }
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
    commands: Commands,
    game: ResMut<Game>,
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

    if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight) {
        let bullet_position = Position::new(player_position.x + 1, player_position.y - 1);
        spawn_bullet(commands, game, &bullet_position);
    }
}

fn spawn_bullet(mut commands: Commands, game: ResMut<Game>, bullet_position: &Position) {
    commands.spawn((
        Bullet,
        bullet_position.clone(),
        Direction(DIRECTION_UP),
        SpriteBundle {
            texture: game.bullet_handles[DIRECTION_UP].clone(),
            transform: position_to_transform(bullet_position.clone()),
            sprite: create_default_sprite(),
            ..default()
        },
    ));
}

fn move_bullets(
    mut query: Query<
        (
            &mut Position,
            &mut Transform,
            &mut Direction,
            &mut Handle<Image>,
            Entity,
        ),
        With<Bullet>,
    >,
    mut commands: Commands,
    game: ResMut<Game>,
) {
    for (mut pos, mut transform, mut dir, mut handle, entity) in &mut query {
        match dir.0 {
            DIRECTION_LEFT => {
                pos.x -= 1;
            }
            DIRECTION_RIGHT => {
                pos.x += 1;
            }
            DIRECTION_UP => {
                pos.y -= 1;
                if pos.y <= Y_MIN {
                    *dir = Direction(DIRECTION_DOWN);
                    // スプライトを変える
                    *handle = game.bullet_handles[DIRECTION_DOWN].clone();
                }
            }
            DIRECTION_DOWN => {
                pos.y += 1;
                if pos.y > Y_MAX {
                    commands.entity(entity).despawn();
                }
            }
            _ => panic!(),
        }
        *transform = position_to_transform(pos.clone());
    }
}

fn play_shoot_sound(
    mut commands: Commands,
    mut shoot_events: EventReader<ShootEvent>,
    sound: Res<HitSound>,
) {
    if !shoot_events.is_empty() {
        shoot_events.clear();
        commands.spawn(AudioBundle {
            source: sound.0.clone(),
            settings: PlaybackSettings::DESPAWN,
        });
    }
}

fn spawn_target(
    mut commands: Commands,
    query: Query<(&Target, &Position)>,
    asset_server: Res<AssetServer>,
) {
    let position = Position::new(
        rand::thread_rng().gen_range(X_MIN + 1..=X_MAX - 1),
        rand::thread_rng().gen_range(Y_MIN..=15),
    );
    let mut target_count = 0;
    for (_, pos) in &query {
        target_count += 1;
        if *pos == position {
            // 既存のターゲットと重なる場合は生成しない
            return;
        }
    }
    if !(rand::thread_rng().gen_range(0.0..1.0) < 0.07 && target_count < 80) {
        return;
    }
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("image/target.png"),
            transform: position_to_transform(position.clone()),
            sprite: create_default_sprite(),
            ..default()
        },
        position,
        Target,
    ));
}

fn update_score(
    mut commands: Commands,
    mut query: Query<Entity, With<NumberType>>,
    game: ResMut<Game>,
) {
    for entity in &mut query {
        commands.entity(entity).despawn();
    }
    spawn_number(game.hi_score, 18, 0, &mut commands, &game, "HiScore");
    spawn_number(game.score, 32, 0, &mut commands, &game, "Score");
}

fn check_for_bullet_target_collisions(
    mut commands: Commands,
    bullets_query: Query<(&mut Position, Entity), (With<Bullet>, Without<Target>)>,
    targets_query: Query<(&mut Position, Entity), (With<Target>, Without<Bullet>)>,
    mut shoot_events: EventWriter<ShootEvent>,
    mut game: ResMut<Game>,
) {
    for (bullet_pos, bullet_entity) in &bullets_query {
        for (target_pos, target_entity) in &targets_query {
            if bullet_pos == target_pos {
                commands.entity(bullet_entity).despawn();
                commands.entity(target_entity).despawn();
                shoot_events.send_default();
                game.score += 1;
                if game.score > game.hi_score {
                    game.hi_score = game.score;
                }
            }
        }
    }
}

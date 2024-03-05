mod assets;
mod components;
mod events;
mod resources;
mod states;

use std::collections::HashSet;

use assets::*;
use bevy::prelude::*;
use bevy_framepace::Limiter;
use components::*;
use events::*;
use rand::Rng;
use resources::*;
use states::*;

const TITLE: &str = "u235";
const SCREEEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 400.0;
const CELL_SIZE_PX: f32 = 16.0;
const FPS: f64 = 30.0;
// 壁で囲まれた領域のmin/max
const X_MIN: i32 = 2;
const X_MAX: i32 = (SCREEEN_WIDTH / CELL_SIZE_PX) as i32 - 3;
const Y_MIN: i32 = 2;
const Y_MAX: i32 = (SCREEN_HEIGHT / CELL_SIZE_PX) as i32 - 3;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: TITLE.into(),
                    name: Some(TITLE.into()),
                    resolution: (SCREEEN_WIDTH, SCREEN_HEIGHT).into(),
                    cursor: bevy::window::Cursor {
                        visible: false,
                        ..default()
                    },
                    ..default()
                }),
                ..default()
            }),
            bevy_framepace::FramepacePlugin,
        ))
        .insert_state(GameState::InGame)
        .init_resource::<Game>()
        .init_resource::<Textures>()
        .insert_resource(bevy_framepace::FramepaceSettings {
            limiter: Limiter::from_framerate(FPS),
            ..default()
        })
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_event::<HitEvent>()
        .add_event::<CrashEvent>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::InGame), setup_ingame)
        .add_systems(OnExit(GameState::InGame), cleanup_ingame)
        .add_systems(
            Update,
            (
                player_system,
                bullet_system,
                target_spawn_system,
                collision_bullet_target_system,
                collision_bullet_bullet_system,
                collision_player_bullet_system,
                score_system,
                hit_event,
                crash_event,
                bevy::window::close_on_esc,
            )
                .chain()
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            Update,
            (restart_system, bevy::window::close_on_esc)
                .chain()
                .run_if(in_state(GameState::GameOver)),
        )
        .run();
}

fn create_top_left_sprite() -> Sprite {
    Sprite {
        anchor: bevy::sprite::Anchor::TopLeft,
        ..default()
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut textures: ResMut<Textures>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Texture
    textures.back = asset_server.load(IMAGE_BACK);
    textures.bullets[components::Direction::Up.to_i32() as usize] = asset_server.load(IMAGE_UP);
    textures.bullets[components::Direction::Left.to_i32() as usize] = asset_server.load(IMAGE_LEFT);
    textures.bullets[components::Direction::Down.to_i32() as usize] = asset_server.load(IMAGE_DOWN);
    textures.bullets[components::Direction::Right.to_i32() as usize] =
        asset_server.load(IMAGE_RIGHT);
    textures.dust = asset_server.load(IMAGE_DUST);
    textures.numbers = asset_server.load(IMAGE_NUMBERS);
    textures.numbers_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        IMAGE_NUMBERS_TILE_SIZE,
        IMAGE_NUMBERS_TILE_COLUMNS,
        IMAGE_NUMBERS_TILE_ROWS,
        None,
        None,
    ));
    textures.player = asset_server.load(IMAGE_PLAYER);
    textures.target = asset_server.load(IMAGE_TARGET);
    textures.title = asset_server.load(IMAGE_TITLE);
    textures.wall = asset_server.load(IMAGE_WALL);

    // Sound
    commands.insert_resource(HitSound(asset_server.load(SOUND_HIT)));
    commands.insert_resource(CrashSound(asset_server.load(SOUND_CRASH)));
}

fn setup_ingame(
    mut commands: Commands,
    mut game: ResMut<Game>,
    textures: ResMut<Textures>,
    query: Query<(&DespawnOnRestart, Entity)>,
) {
    game.started_count += 1;
    game.reset();

    for (_, entity) in &query {
        commands.entity(entity).despawn();
    }

    // Camera
    // 画面左上がワールド座標(0, 400)、右下が(640, 0)となるようにカメラを移動
    let projection = OrthographicProjection::default();
    commands.spawn((
        DespawnOnRestart,
        Camera2dBundle {
            transform: Transform::from_xyz(
                SCREEEN_WIDTH / 2.0,
                SCREEN_HEIGHT / 2.0,
                projection.far - 1.0,
            ),
            projection: projection,
            ..default()
        },
    ));

    let sprite: Sprite = create_top_left_sprite();

    // Player
    let player_position = Position::new(18, Y_MAX);
    commands.spawn((
        Player,
        DespawnOnRestart,
        player_position.clone(),
        SpriteBundle {
            texture: textures.player.clone(),
            transform: position_to_transform(player_position.clone()),
            sprite: sprite.clone(),
            ..default()
        },
    ));

    // Walls
    let mut spawn_wall = |x, y| {
        commands.spawn((
            DespawnOnRestart,
            SpriteBundle {
                texture: textures.wall.clone(),
                transform: position_to_transform(Position::new(x, y)),
                sprite: sprite.clone(),
                ..default()
            },
        ));
    };
    for y in 1..=Y_MAX {
        spawn_wall(1, y);
        spawn_wall(X_MAX + 1, y);
    }
    for x in 1..(X_MAX + 1) {
        spawn_wall(x, 1);
    }

    // Back
    for i in (X_MIN - 2)..=(X_MAX + 2) {
        commands.spawn((
            DespawnOnRestart,
            SpriteBundle {
                texture: textures.back.clone(),
                transform: position_to_transform(Position::new(i, Y_MAX + 1)),
                sprite: sprite.clone(),
                ..default()
            },
        ));
    }

    // Title
    commands.spawn((
        DespawnOnRestart,
        SpriteBundle {
            texture: textures.title.clone(),
            transform: position_to_transform(Position::new(1, 0)),
            sprite: sprite.clone(),
            ..default()
        },
    ));

    // Score, HiScore
    let textures: &Res<'_, Textures> = &textures.into();
    spawn_number(game.hi_score, 18, 0, &mut commands, textures, "HiScore");
    spawn_number(game.score, 32, 0, &mut commands, textures, "Score");
}

fn cleanup_ingame() {}

fn spawn_number(
    num: i32,
    cx: i32,
    cy: i32,
    commands: &mut Commands,
    textures: &Res<Textures>,
    label: &'static str,
) {
    let text = format!("{:8}", num);
    let mut numbers_pos = position_to_transform(Position::new(cx, cy));
    numbers_pos.translation.z = 2.0; // titleより手前
    for i in 0..8 {
        let byte = text.as_bytes()[i];
        let mut spawn_num = |atlas_index, visibility| {
            commands.spawn((
                NumberType(label, i),
                DespawnOnRestart,
                SpriteSheetBundle {
                    texture: textures.numbers.clone(),
                    atlas: TextureAtlas {
                        layout: textures.numbers_layout.clone(),
                        index: atlas_index,
                    },
                    transform: numbers_pos,
                    visibility: visibility,
                    sprite: create_top_left_sprite(),
                    ..default()
                },
            ));
        };
        if 0x30 <= byte && byte <= 0x39 {
            spawn_num((byte - 0x30) as usize, Visibility::Visible);
        } else {
            spawn_num(0, Visibility::Hidden);
        }
        numbers_pos.translation.x += IMAGE_NUMBERS_TILE_SIZE.x;
    }
}

fn score_system(
    mut query: Query<(&NumberType, &mut TextureAtlas, &mut Visibility)>,
    game: ResMut<Game>,
) {
    for (number_type, mut texture_atlas, mut visibility) in &mut query {
        let i = number_type.1;
        let num;
        if number_type.0 == "Score" {
            num = game.score;
        } else if number_type.0 == "HiScore" {
            num = game.hi_score;
        } else {
            panic!();
        }

        let text = format!("{:8}", num);
        let byte = text.as_bytes()[i];
        if 0x30 <= byte && byte <= 0x39 {
            texture_atlas.index = (byte - 0x30) as usize;
            *visibility = Visibility::Visible;
        } else {
            texture_atlas.index = 0;
            *visibility = Visibility::Hidden;
        }
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

fn player_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Position), With<Player>>,
    mut commands: Commands,
    textures: Res<Textures>,
) {
    for (mut transform, mut position) in &mut query {
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            if position.x > X_MIN {
                position.x = position.x - 1;
            }
        }

        if keyboard_input.pressed(KeyCode::ArrowRight) {
            if position.x < X_MAX - 2 {
                position.x = position.x + 1;
            }
        }
        transform.translation = position_to_transform(position.clone()).translation;

        if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight)
        {
            let bullet_position = Position::new(position.x + 1, position.y - 1);
            spawn_bullet(
                &mut commands,
                &textures,
                &bullet_position,
                components::Direction::Up,
                false,
            );
        }
    }
}

fn restart_system(
    mut next_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::InGame);
    }
}

fn spawn_bullet(
    commands: &mut Commands,
    textures: &Res<Textures>,
    bullet_position: &Position,
    direction: components::Direction,
    is_dust: bool,
) {
    commands.spawn((
        Bullet,
        DespawnOnRestart,
        bullet_position.clone(),
        direction.clone(),
        SpriteBundle {
            texture: if is_dust {
                textures.dust.clone()
            } else {
                textures.bullets[direction.to_i32() as usize].clone()
            },
            transform: position_to_transform(bullet_position.clone()),
            sprite: create_top_left_sprite(),
            ..default()
        },
    ));
}

fn bullet_system(
    mut query: Query<
        (
            &mut Position,
            &mut Transform,
            &mut components::Direction,
            &mut Handle<Image>,
            Entity,
        ),
        With<Bullet>,
    >,
    mut commands: Commands,
    textures: Res<Textures>,
) {
    for (mut pos, mut transform, mut dir, mut handle, entity) in &mut query {
        match *dir {
            components::Direction::Left => {
                pos.x -= 1;
                if pos.x <= X_MIN {
                    *dir = dir.opposite();
                    *handle = textures.bullets[dir.to_i32() as usize].clone();
                }
            }
            components::Direction::Right => {
                pos.x += 1;
                if pos.x >= X_MAX {
                    *dir = dir.opposite();
                    *handle = textures.bullets[dir.to_i32() as usize].clone();
                }
            }
            components::Direction::Up => {
                pos.y -= 1;
                if pos.y <= Y_MIN {
                    *dir = dir.opposite();
                    // スプライトを変える
                    *handle = textures.bullets[dir.to_i32() as usize].clone();
                }
            }
            components::Direction::Down => {
                pos.y += 1;
                if pos.y > Y_MAX {
                    commands.entity(entity).despawn();
                }
            }
        }
        *transform = position_to_transform(pos.clone());
    }
}

fn hit_event(mut commands: Commands, mut hit_events: EventReader<HitEvent>, sound: Res<HitSound>) {
    if !hit_events.is_empty() {
        hit_events.clear();
        commands.spawn(AudioBundle {
            source: sound.0.clone(),
            settings: PlaybackSettings::DESPAWN,
        });
    }
}

fn crash_event(
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    mut crash_events: EventReader<CrashEvent>,
    sound: Res<CrashSound>,
    textures: Res<Textures>,
) {
    if !crash_events.is_empty() {
        for event in crash_events.read() {
            let position = event.pos.clone();
            commands.spawn(AudioBundle {
                source: sound.0.clone(),
                settings: PlaybackSettings::DESPAWN,
            });
            for i in 0..3 {
                commands.spawn((
                    DespawnOnRestart,
                    SpriteBundle {
                        texture: textures.dust.clone(),
                        sprite: create_top_left_sprite(),
                        transform: position_to_transform(Position::new(position.x + i, position.y)),
                        ..default()
                    },
                ));
            }
            commands.spawn((
                DespawnOnRestart,
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgba(1.0, 0.0, 0.0, 0.5),
                        anchor: bevy::sprite::Anchor::BottomLeft,
                        custom_size: Some(Vec2::new(SCREEEN_WIDTH, SCREEN_HEIGHT)),
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 0.0, 3.0),
                    ..default()
                },
            ));
            break;
        }
        crash_events.clear();
        next_state.set(GameState::GameOver);
    }
}

fn target_spawn_system(
    mut commands: Commands,
    query: Query<(&Target, &Position)>,
    textures: Res<Textures>,
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
        Target,
        DespawnOnRestart,
        SpriteBundle {
            texture: textures.target.clone(),
            transform: position_to_transform(position.clone()),
            sprite: create_top_left_sprite(),
            ..default()
        },
        position,
    ));
}

fn collision_bullet_target_system(
    mut commands: Commands,
    bullets_query: Query<(&Position, Entity), (With<Bullet>, Without<Target>)>,
    targets_query: Query<(&Position, Entity), (With<Target>, Without<Bullet>)>,
    mut hit_events: EventWriter<HitEvent>,
    mut game: ResMut<Game>,
    textures: Res<Textures>,
) {
    let mut despawned_entities: HashSet<Entity> = HashSet::new();

    for (bullet_pos, bullet_entity) in &bullets_query {
        if despawned_entities.contains(&bullet_entity) {
            continue;
        }
        for (target_pos, target_entity) in &targets_query {
            if despawned_entities.contains(&target_entity) {
                continue;
            }
            if bullet_pos == target_pos {
                commands.entity(bullet_entity).despawn();
                commands.entity(target_entity).despawn();
                despawned_entities.insert(bullet_entity);
                despawned_entities.insert(target_entity);
                hit_events.send_default();
                game.score += 1000;
                if game.score > game.hi_score {
                    game.hi_score = game.score;
                }
                for dir in components::Direction::all() {
                    spawn_bullet(
                        &mut commands,
                        &textures,
                        &dir.neighbor(bullet_pos.clone()),
                        dir.clone(),
                        dir == components::Direction::Down,
                    );
                }
            }
        }
    }
}

fn collision_bullet_bullet_system(
    mut commands: Commands,
    bullets_query0: Query<
        (&Position, &components::Direction, Entity),
        (With<Bullet>, Without<Target>),
    >,
    bullets_query1: Query<
        (&Position, &components::Direction, Entity),
        (With<Bullet>, Without<Target>),
    >,
) {
    let mut despawned_entities: HashSet<Entity> = HashSet::new();

    for (bullet_pos0, dir0, bullet_entity0) in &bullets_query0 {
        if despawned_entities.contains(&bullet_entity0) {
            continue;
        }
        for (bullet_pos1, dir1, bullet_entity1) in &bullets_query1 {
            if despawned_entities.contains(&bullet_entity1) {
                continue;
            }
            if bullet_pos0 == bullet_pos1
                && bullet_entity0 != bullet_entity1
                && ((*dir0 == components::Direction::Left && *dir1 == components::Direction::Right)
                    || (*dir0 == components::Direction::Right
                        && *dir1 == components::Direction::Left))
            {
                commands.entity(bullet_entity0).despawn();
                commands.entity(bullet_entity1).despawn();
                despawned_entities.insert(bullet_entity0);
                despawned_entities.insert(bullet_entity1);
            }
        }
    }
}

fn collision_player_bullet_system(
    mut commands: Commands,
    players_query: Query<(&Position, Entity), With<Player>>,
    bullets_query: Query<(&Position, Entity), (With<Bullet>, Without<Target>)>,
    mut crash_events: EventWriter<CrashEvent>,
) {
    let mut despawned_entities: HashSet<Entity> = HashSet::new();

    for (player_pos, player_entity) in &players_query {
        if despawned_entities.contains(&player_entity) {
            return;
        }
        for (bullet_pos, bullet_entity) in &bullets_query {
            if despawned_entities.contains(&bullet_entity) {
                return;
            }
            if bullet_pos.y == player_pos.y
                && (player_pos.x <= bullet_pos.x && bullet_pos.x <= player_pos.x + 2)
            {
                commands.entity(player_entity).despawn();
                commands.entity(bullet_entity).despawn();
                despawned_entities.insert(player_entity);
                despawned_entities.insert(bullet_entity);
                crash_events.send(CrashEvent {
                    pos: player_pos.clone(),
                });
            }
        }
    }
}

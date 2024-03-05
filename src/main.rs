use bevy::prelude::*;
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

mod assets {
    use bevy::prelude::*;

    pub const IMAGE_BACK: &str = "image/back.png";
    pub const IMAGE_DOWN: &str = "image/down.png";
    pub const IMAGE_DUST: &str = "image/dust.png";
    pub const IMAGE_LEFT: &str = "image/left.png";
    pub const IMAGE_NUMBERS: &str = "image/numbers.png";
    pub const IMAGE_NUMBERS_TILE_SIZE: Vec2 = Vec2::new(8.0, 16.0);
    pub const IMAGE_NUMBERS_TILE_COLUMNS: usize = 10;
    pub const IMAGE_NUMBERS_TILE_ROWS: usize = 1;
    pub const IMAGE_PLAYER: &str = "image/player.png";
    pub const IMAGE_RIGHT: &str = "image/right.png";
    pub const IMAGE_TARGET: &str = "image/target.png";
    pub const IMAGE_TITLE: &str = "image/title.png";
    pub const IMAGE_UP: &str = "image/up.png";
    pub const IMAGE_WALL: &str = "image/wall.png";
    pub const SOUND_CRASH: &str = "sound/crash.wav";
    pub const SOUND_HIT: &str = "sound/hit.wav";
}

// このmarkerをつけたComponentはリスタート時にdespawnされる
// https://www.reddit.com/r/bevy/comments/17er37y/comment/k65wjdn/?utm_source=share&utm_medium=web3x&utm_name=web3xcss&utm_term=1&utm_content=share_button
#[derive(Component)]
struct DespawnOnRestart;

#[derive(Debug, Default, Clone, Eq, PartialEq, Component)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn add(&self, x: i32, y: i32) -> Self {
        Self {
            x: self.x + x,
            y: self.y + y,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Component)]
enum Direction {
    Up,
    Left,
    Down,
    Right,
}

impl Direction {
    pub fn all() -> [Self; 4] {
        [
            Direction::Up,
            Direction::Left,
            Direction::Down,
            Direction::Right,
        ]
    }

    pub fn from_i32(n: i32) -> Self {
        match n {
            0 => Direction::Up,
            1 => Direction::Left,
            2 => Direction::Down,
            3 => Direction::Right,
            _ => panic!(),
        }
    }

    pub fn to_i32(&self) -> i32 {
        match self {
            Direction::Up => 0,
            Direction::Left => 1,
            Direction::Down => 2,
            Direction::Right => 3,
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Left => Direction::Right,
            Direction::Down => Direction::Up,
            Direction::Right => Direction::Left,
        }
    }

    pub fn neighbor(&self, pos: Position) -> Position {
        match self {
            Direction::Up => pos.add(0, -1),
            Direction::Left => pos.add(-1, 0),
            Direction::Down => pos.add(0, 1),
            Direction::Right => pos.add(1, 0),
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct Target;

#[derive(Component)]
struct NumberType(&'static str, usize);

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    InGame,
    GameOver,
}

#[derive(Resource)]
struct HitSound(Handle<AudioSource>);

#[derive(Resource)]
struct CrashSound(Handle<AudioSource>);

#[derive(Resource, Default)]
struct Game {
    score: i32,
    hi_score: i32,
    bullet_handles: [Handle<Image>; 4],
    texture_atlas_layout: Handle<TextureAtlasLayout>,
    number_texture: Handle<Image>,
    dust_texture: Handle<Image>,
}

#[derive(Event, Default)]
struct HitEvent;

#[derive(Event, Default)]
struct CrashEvent {
    pos: Position,
}

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
                update_player,
                update_bullets,
                spawn_target,
                check_for_bullet_target_collisions,
                check_for_bullet_bullet_collisions,
                check_for_player_bullet_collisions,
                update_score,
                bevy::window::close_on_esc,
                play_hit_sound,
                crash_event,
            )
                .chain()
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            Update,
            (restart, bevy::window::close_on_esc)
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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut game: ResMut<Game>) {
    game.bullet_handles[Direction::Up.to_i32() as usize] = asset_server.load(assets::IMAGE_UP);
    game.bullet_handles[Direction::Left.to_i32() as usize] = asset_server.load(assets::IMAGE_LEFT);
    game.bullet_handles[Direction::Down.to_i32() as usize] = asset_server.load(assets::IMAGE_DOWN);
    game.bullet_handles[Direction::Right.to_i32() as usize] =
        asset_server.load(assets::IMAGE_RIGHT);
    game.dust_texture = asset_server.load(assets::IMAGE_DUST);

    // Sound
    commands.insert_resource(HitSound(asset_server.load(assets::SOUND_HIT)));
    commands.insert_resource(CrashSound(asset_server.load(assets::SOUND_CRASH)));
}

fn setup_ingame(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game: ResMut<Game>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    query: Query<(&DespawnOnRestart, Entity)>,
) {
    println!("setup_ingame");

    for (_, entity) in &query {
        commands.entity(entity).despawn();
    }

    // Camera
    // ワールド座標は画面左上を(0, 400)、右下を(640, 0)とする
    commands.spawn((
        DespawnOnRestart,
        Camera2dBundle {
            transform: Transform::from_xyz(SCREEEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0, 999.0),
            projection: OrthographicProjection {
                scaling_mode: bevy::render::camera::ScalingMode::WindowSize(1.0),
                ..default()
            },
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
            texture: asset_server.load(assets::IMAGE_PLAYER),
            transform: position_to_transform(player_position.clone()),
            sprite: sprite.clone(),
            ..default()
        },
    ));

    // Walls
    for y in 1..=Y_MAX {
        commands.spawn((
            DespawnOnRestart,
            SpriteBundle {
                texture: asset_server.load(assets::IMAGE_WALL),
                transform: position_to_transform(Position::new(1, y)),
                sprite: sprite.clone(),
                ..default()
            },
        ));
        commands.spawn((
            DespawnOnRestart,
            SpriteBundle {
                texture: asset_server.load(assets::IMAGE_WALL),
                transform: position_to_transform(Position::new(X_MAX + 1, y)),
                sprite: sprite.clone(),
                ..default()
            },
        ));
    }
    for x in 1..(X_MAX + 1) {
        commands.spawn((
            DespawnOnRestart,
            SpriteBundle {
                texture: asset_server.load(assets::IMAGE_WALL),
                transform: position_to_transform(Position::new(x, 1)),
                sprite: sprite.clone(),
                ..default()
            },
        ));
    }

    // Back
    for i in (X_MIN - 2)..=(X_MAX + 2) {
        commands.spawn((
            DespawnOnRestart,
            SpriteBundle {
                texture: asset_server.load(assets::IMAGE_BACK),
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
            texture: asset_server.load(assets::IMAGE_TITLE),
            transform: position_to_transform(Position::new(1, 0)),
            sprite: sprite.clone(),
            ..default()
        },
    ));

    let texture: Handle<Image> = asset_server.load(assets::IMAGE_NUMBERS);
    let layout = TextureAtlasLayout::from_grid(
        assets::IMAGE_NUMBERS_TILE_SIZE,
        assets::IMAGE_NUMBERS_TILE_COLUMNS,
        assets::IMAGE_NUMBERS_TILE_ROWS,
        None,
        None,
    );
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    game.texture_atlas_layout = texture_atlas_layout;
    game.number_texture = texture;

    // Score, HiScore
    spawn_number(game.hi_score, 18, 0, &mut commands, &mut game, "HiScore");
    spawn_number(game.score, 32, 0, &mut commands, &mut game, "Score");
}

fn cleanup_ingame(mut commands: Commands) {
    println!("cleanup_ingame");
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
                NumberType(label, i),
                DespawnOnRestart,
                SpriteSheetBundle {
                    texture: game.number_texture.clone(),
                    atlas: TextureAtlas {
                        layout: game.texture_atlas_layout.clone(),
                        index: index,
                    },
                    transform: numbers_pos,
                    visibility: Visibility::Visible,
                    sprite: create_top_left_sprite(),
                    ..default()
                },
            ));
        }
        numbers_pos.translation.x += assets::IMAGE_NUMBERS_TILE_SIZE.x;
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

fn update_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Position), With<Player>>,
    mut commands: Commands,
    game: ResMut<Game>,
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
            spawn_bullet(&mut commands, &game, &bullet_position, Direction::Up, false);
        }
    }
}

fn restart(
    mut next_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::InGame);
    }
}

fn spawn_bullet(
    commands: &mut Commands,
    game: &ResMut<Game>,
    bullet_position: &Position,
    direction: Direction,
    is_dust: bool,
) {
    commands.spawn((
        Bullet,
        DespawnOnRestart,
        bullet_position.clone(),
        direction.clone(),
        SpriteBundle {
            texture: if is_dust {
                game.dust_texture.clone()
            } else {
                game.bullet_handles[direction.to_i32() as usize].clone()
            },
            transform: position_to_transform(bullet_position.clone()),
            sprite: create_top_left_sprite(),
            ..default()
        },
    ));
}

fn update_bullets(
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
        match *dir {
            Direction::Left => {
                pos.x -= 1;
                if pos.x <= X_MIN {
                    *dir = dir.opposite();
                    *handle = game.bullet_handles[dir.to_i32() as usize].clone();
                }
            }
            Direction::Right => {
                pos.x += 1;
                if pos.x >= X_MAX {
                    *dir = dir.opposite();
                    *handle = game.bullet_handles[dir.to_i32() as usize].clone();
                }
            }
            Direction::Up => {
                pos.y -= 1;
                if pos.y <= Y_MIN {
                    *dir = dir.opposite();
                    // スプライトを変える
                    *handle = game.bullet_handles[dir.to_i32() as usize].clone();
                }
            }
            Direction::Down => {
                pos.y += 1;
                if pos.y > Y_MAX {
                    commands.entity(entity).despawn();
                }
            }
        }
        *transform = position_to_transform(pos.clone());
    }
}

fn play_hit_sound(
    mut commands: Commands,
    mut hit_events: EventReader<HitEvent>,
    sound: Res<HitSound>,
) {
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
    game: Res<Game>,
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
                        texture: game.dust_texture.clone(),
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
        Target,
        DespawnOnRestart,
        SpriteBundle {
            texture: asset_server.load(assets::IMAGE_TARGET),
            transform: position_to_transform(position.clone()),
            sprite: create_top_left_sprite(),
            ..default()
        },
        position,
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
    bullets_query: Query<(&Position, Entity), (With<Bullet>, Without<Target>)>,
    targets_query: Query<(&Position, Entity), (With<Target>, Without<Bullet>)>,
    mut hit_events: EventWriter<HitEvent>,
    mut game: ResMut<Game>,
) {
    for (bullet_pos, bullet_entity) in &bullets_query {
        for (target_pos, target_entity) in &targets_query {
            if bullet_pos == target_pos {
                commands.entity(bullet_entity).despawn();
                commands.entity(target_entity).despawn();
                hit_events.send_default();
                game.score += 1000;
                if game.score > game.hi_score {
                    game.hi_score = game.score;
                }
                for dir in Direction::all() {
                    spawn_bullet(
                        &mut commands,
                        &game,
                        &dir.neighbor(bullet_pos.clone()),
                        dir.clone(),
                        dir == Direction::Down,
                    );
                }
            }
        }
    }
}

fn check_for_bullet_bullet_collisions(
    mut commands: Commands,
    bullets_query0: Query<(&Position, &Direction, Entity), (With<Bullet>, Without<Target>)>,
    bullets_query1: Query<(&Position, &Direction, Entity), (With<Bullet>, Without<Target>)>,
) {
    for (bullet_pos0, dir0, bullet_entity0) in &bullets_query0 {
        for (bullet_pos1, dir1, bullet_entity1) in &bullets_query1 {
            if bullet_pos0 == bullet_pos1
                && bullet_entity0 != bullet_entity1
                && ((*dir0 == Direction::Left && *dir1 == Direction::Right)
                    || (*dir0 == Direction::Right && *dir1 == Direction::Left))
            {
                commands.entity(bullet_entity0).despawn();
                commands.entity(bullet_entity1).despawn();
            }
        }
    }
}

fn check_for_player_bullet_collisions(
    mut commands: Commands,
    players_query: Query<(&Position, Entity), With<Player>>,
    bullets_query: Query<(&Position, Entity), (With<Bullet>, Without<Target>)>,
    mut crash_events: EventWriter<CrashEvent>,
) {
    for (player_pos, player_entity) in &players_query {
        for (bullet_pos, bullet_entity) in &bullets_query {
            if bullet_pos.y == player_pos.y
                && (player_pos.x <= bullet_pos.x && bullet_pos.x <= player_pos.x + 2)
            {
                commands.entity(player_entity).despawn();
                commands.entity(bullet_entity).despawn();
                crash_events.send(CrashEvent {
                    pos: player_pos.clone(),
                });
            }
        }
    }
}

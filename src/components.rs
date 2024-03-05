use bevy::prelude::*;

// このmarkerをつけたComponentはリスタート時にdespawnされる
// https://www.reddit.com/r/bevy/comments/17er37y/comment/k65wjdn/?utm_source=share&utm_medium=web3x&utm_name=web3xcss&utm_term=1&utm_content=share_button
#[derive(Component)]
pub struct DespawnOnRestart;

#[derive(Debug, Default, Clone, Eq, PartialEq, Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
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
pub enum Direction {
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

    #[allow(unused)]
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
pub struct Player;

#[derive(Component)]
pub struct Bullet;

#[derive(Component)]
pub struct Target;

#[derive(Component)]
pub struct NumberType(pub &'static str, pub usize);

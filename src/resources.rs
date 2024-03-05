use bevy::prelude::*;

#[derive(Resource)]
pub struct HitSound(pub Handle<AudioSource>);

#[derive(Resource)]
pub struct CrashSound(pub Handle<AudioSource>);

#[derive(Resource, Default)]
pub struct Game {
    pub score: i32,
    pub hi_score: i32,
}

impl Game {
    pub fn reset(&mut self) {
        self.score = 0;
    }
}

#[derive(Resource, Default)]
pub struct Textures {
    pub back: Handle<Image>,
    pub bullets: [Handle<Image>; 4],
    pub dust: Handle<Image>,
    pub numbers: Handle<Image>,
    pub numbers_layout: Handle<TextureAtlasLayout>,
    pub player: Handle<Image>,
    pub target: Handle<Image>,
    pub title: Handle<Image>,
    pub wall: Handle<Image>,
}

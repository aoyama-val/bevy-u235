use crate::components::*;
use bevy::prelude::*;

#[derive(Event, Default)]
pub struct HitEvent;

#[derive(Event, Default)]
pub struct CrashEvent {
    pub pos: Position,
}

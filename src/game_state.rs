use rand::prelude::*;

use crate::map::Vec2f;
use crate::entities::{GuidedMissile, Tank};

/// Everyting that changes during the game
/// and might need to be taken back during frame interpolation / reconciliation.
#[derive(Debug, Clone)]
pub struct GameState {
    pub rng: SmallRng,
    /// This frame's time in seconds
    pub frame_time: f64,
    pub input: Input,
    pub cur_weapon: usize,
    pub gm: GuidedMissile,
    pub tank: Tank,
    pub pe: PlayerEntity,
    pub explosions: Vec<(Vec2f, i32)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerEntity {
    GuidedMissile,
    Tank,
}

#[derive(Debug, Clone, Default)]
pub struct Input {
    pub left: f64,
    pub right: f64,
    pub up: f64,
    pub down: f64,
    pub change_weapon: bool,
    pub fire: bool,
}

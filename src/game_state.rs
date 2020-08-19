use rand::prelude::*;

use wasm_bindgen::prelude::*;

use crate::entities::{GuidedMissile, Tank};
use crate::map::Vec2f;

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

#[wasm_bindgen]
#[derive(Debug, Clone, Default)]
pub struct Input {
    pub left: f64,
    pub right: f64,
    pub up: f64,
    pub down: f64,
    pub change_weapon: bool,
    pub fire: bool,
}

#[wasm_bindgen]
impl Input {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
}

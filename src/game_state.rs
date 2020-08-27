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
    pub railguns: Vec<(Vec2f, Vec2f)>,
    pub gm: GuidedMissile,
    pub tank: Tank,
    pub pe: PlayerEntity,
    pub explosions: Vec<Explosion>,
}

#[derive(Debug, Clone)]
pub struct Explosion {
    pub pos: Vec2f,
    pub size: f64,
    pub start_time: f64,
}

impl Explosion {
    pub fn new(pos: Vec2f, size: f64, start_time: f64) -> Self {
        Self {
            pos,
            size,
            start_time,
        }
    }
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
    pub prev_weapon: bool,
    pub next_weapon: bool,
    pub fire: bool,
}

#[wasm_bindgen]
impl Input {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
}

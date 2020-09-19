use rand::prelude::*;

use wasm_bindgen::prelude::*;

use crate::map::Vec2f;
use crate::{
    entities::{GuidedMissile, Tank},
    weapons::Weapon,
};

/// Everyting that changes during the game
/// and might need to be taken back during frame interpolation / reconciliation.
#[derive(Debug, Clone)]
pub(crate) struct GameState {
    pub(crate) rng: SmallRng,
    /// This frame's time in seconds
    pub(crate) frame_time: f64,
    pub(crate) input: Input,
    pub(crate) cur_weapon: Weapon,
    pub(crate) railguns: Vec<(Vec2f, Vec2f)>,
    pub(crate) gm: GuidedMissile,
    pub(crate) tank: Tank,
    pub(crate) pe: PlayerEntity,
    pub(crate) explosions: Vec<Explosion>,
}

#[derive(Debug, Clone)]
pub(crate) struct Explosion {
    pub(crate) pos: Vec2f,
    pub(crate) scale: f64,
    pub(crate) start_time: f64,
    pub(crate) bfg: bool,
}

impl Explosion {
    pub(crate) fn new(pos: Vec2f, scale: f64, start_time: f64, bfg: bool) -> Self {
        Self {
            pos,
            scale,
            start_time,
            bfg,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlayerEntity {
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
    pub turret_left: bool,
    pub turret_right: bool,
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

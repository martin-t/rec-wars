use rand::prelude::*;
use thunderdome::{Arena, Index};
use wasm_bindgen::prelude::*;

use crate::{
    entities::{Ai, Player, Projectile, Vehicle},
    map::Vec2f,
};

/// Things that change during the game
/// and might need to be taken back during frame interpolation / reconciliation.
///
/// TODO How to do frame interpolation / server reconcilliation?
/// Ralith (hecs author) says to make all components a Vec but that requires all code to be aware of interpolation.
/// What does veloren do?
#[derive(Debug, Clone)]
pub(crate) struct GameState {
    pub(crate) rng: SmallRng,
    /// This frame's time in seconds
    pub(crate) frame_time: f64,
    /// Delta time since last frame in seconds
    pub(crate) dt: f64,
    pub(crate) railguns: Vec<(Vec2f, Vec2f)>,
    pub(crate) bfg_beams: Vec<(Vec2f, Vec2f)>,
    pub(crate) player_handle: Index,
    pub(crate) explosions: Vec<Explosion>,
    pub(crate) ais: Arena<Ai>,
    pub(crate) players: Arena<Player>,
    pub(crate) vehicles: Arena<Vehicle>,
    pub(crate) projectiles: Arena<Projectile>,
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

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, Default)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub turret_left: bool,
    pub turret_right: bool,
    pub prev_weapon: bool,
    pub next_weapon: bool,
    pub fire: bool,
    pub mine: bool,
    pub self_destruct: bool,
    pub horn: bool,
    pub chat: bool,
}

#[wasm_bindgen]
impl Input {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_up() -> Self {
        Self {
            up: true,
            ..Self::default()
        }
    }

    pub(crate) fn right_left(&self) -> f64 {
        self.right as i32 as f64 - self.left as i32 as f64
    }

    pub(crate) fn up(&self) -> f64 {
        self.up as i32 as f64
    }

    pub(crate) fn down(&self) -> f64 {
        self.down as i32 as f64
    }

    /// Subset of inputs to control the missile
    pub(crate) fn missile_while_guiding(&self) -> Self {
        Self {
            up: true,
            down: false,
            ..*self
        }
    }

    /// Subset of inputs to control the vehicle while guiding a missile
    pub(crate) fn vehicle_while_guiding(&self) -> Self {
        // TODO what exactly is allowed? mines? make configurable
        Self {
            left: false,
            right: false,
            up: false,
            down: false,
            ..*self
        }
    }
}

pub(crate) trait ArenaExt {
    fn iter_handles(&self) -> Vec<Index>;
}

impl<T> ArenaExt for Arena<T> {
    fn iter_handles(&self) -> Vec<Index> {
        self.iter().map(|(handle, _)| handle).collect()
    }
}

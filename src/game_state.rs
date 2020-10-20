use legion::Entity;

use rand::prelude::*;

use wasm_bindgen::prelude::*;

use crate::map::Vec2f;

/// Everyting that changes during the game
/// and might need to be taken back during frame interpolation / reconciliation.
/// TODO this is not accurate if ECS is outside GameState
#[derive(Debug, Clone)]
pub(crate) struct GameState {
    pub(crate) rng: SmallRng,
    /// This frame's time in seconds
    pub(crate) frame_time: f64,
    /// Delta time since last frame in seconds
    pub(crate) dt: f64,
    pub(crate) input: Input,
    pub(crate) railguns: Vec<(Vec2f, Vec2f)>,
    pub(crate) bfg_beams: Vec<(Vec2f, Vec2f)>,
    /// Player entity - the vehicle
    pub(crate) player_entity: Entity,
    pub(crate) guided_missile: Option<Entity>,
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

#[derive(Debug, Clone)]
pub(crate) struct Damage {
    attacker: Entity,
    dmg: f64,
    victim: Entity,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Default)]
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
    pub(crate) fn guided_missile(&self) -> Self {
        Self {
            up: true,
            down: false,
            ..*self
        }
    }

    /// Subset of inputs to control the vehicle while guiding a missile.
    pub(crate) fn vehicle_while_guiding(&self) -> Self {
        Self {
            left: false,
            right: false,
            up: false,
            down: false,
            ..*self
        }
    }
}

pub(crate) const EMPTY_INPUT: Input = Input {
    // Can't use Input::default() in constants.
    left: false,
    right: false,
    up: false,
    down: false,
    turret_left: false,
    turret_right: false,
    prev_weapon: false,
    next_weapon: false,
    fire: false,
    mine: false,
    self_destruct: false,
    horn: false,
    chat: false,
};

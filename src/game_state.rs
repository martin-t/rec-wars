use std::fmt::{self, Debug, Formatter};

use fnv::FnvHashMap;
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
    pub(crate) rail_beams: Vec<RailBeam>,
    /// Map of projectile handles to vehicle handles.
    /// Prevents rail hitting the same vehicle twice
    /// when one segment ends inside the hitbox and the next starts inside it the next frame.
    /// This can for now only happen with railguns since all other projectiles get removed on hit.
    /// TODO This is still not perfect since one segment can hit multiple vehicles in any order
    /// and there's no guarantee the last vehicle is the one where the beam ends.
    /// LATER This is a can of worms:
    ///     1) Make sure (add test) one beam can kill the player and hit him again if he's unlucky enough to respawn in its path.
    ///     2) Remove the entry after the projectile exist the hitbox - e.g. guided missiles that can pass through several times.
    ///     3) Make sure the HashMap doesn't grow indefinitely in case we forgot to remove in some cases.
    pub(crate) rail_hits: FnvHashMap<Index, Index>,
    pub(crate) bfg_beams: Vec<(Vec2f, Vec2f)>,
    pub(crate) player_handle: Index,
    pub(crate) explosions: Vec<Explosion>,
    pub(crate) ais: Arena<Ai>,
    pub(crate) players: Arena<Player>,
    pub(crate) vehicles: Arena<Vehicle>,
    pub(crate) projectiles: Arena<Projectile>,
}

#[derive(Debug, Clone)]
pub(crate) struct RailBeam {
    pub(crate) begin: Vec2f,
    pub(crate) end: Vec2f,
    pub(crate) start_time: f64,
}

impl RailBeam {
    pub(crate) fn new(begin: Vec2f, end: Vec2f, start_time: f64) -> Self {
        Self {
            begin,
            end,
            start_time,
        }
    }
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
#[derive(Clone, Copy, Default)]
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
        // Original RW allowed everything except movement.
        Self {
            left: false,
            right: false,
            up: false,
            down: false,
            ..*self
        }
    }
}

impl Debug for Input {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Input {{ ")?;
        if self.left {
            write!(f, "left ")?;
        }
        if self.right {
            write!(f, "right ")?;
        }
        if self.up {
            write!(f, "up ")?;
        }
        if self.down {
            write!(f, "down ")?;
        }
        if self.turret_left {
            write!(f, "turret_left ")?;
        }
        if self.turret_right {
            write!(f, "turret_right ")?;
        }
        if self.prev_weapon {
            write!(f, "prev_weapon ")?;
        }
        if self.next_weapon {
            write!(f, "next_weapon ")?;
        }
        if self.fire {
            write!(f, "fire ")?;
        }
        if self.mine {
            write!(f, "mine ")?;
        }
        if self.self_destruct {
            write!(f, "self_destruct ")?;
        }
        if self.horn {
            write!(f, "horn ")?;
        }
        if self.chat {
            write!(f, "chat ")?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

pub(crate) trait ArenaExt {
    /// Collect the handles (`thunderdome::Index`) into a `Vec`.
    ///
    /// This is borrowchk dance to allow iterating through the collection without keeping the arena borrowed.
    /// You can reborrow each iteration of the loop by indexing the arena using the handle
    /// and release the borrow if you need to pass the arena (or usually whole `GameState`) into another function.
    fn iter_handles(&self) -> Vec<Index>;
}

impl<T> ArenaExt for Arena<T> {
    fn iter_handles(&self) -> Vec<Index> {
        self.iter().map(|(handle, _)| handle).collect()
    }
}

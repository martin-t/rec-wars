use rand::prelude::*;

use crate::cvars::Cvars;
use crate::data::{Map, Vec2f};

/// Returns (pos, angle).
pub fn random_spawn_pos(rng: &mut SmallRng, map: &Map) -> (Vec2f, f64) {
    let r = rng.gen_range(0, map.spawns().len());
    let index = map.spawns()[r];
    let pos = map.tile_center(index);
    let angle = map[index].angle;
    (pos, angle)
}

#[derive(Debug, Clone)]
pub struct GuidedMissile {
    pub pos: Vec2f,
    pub vel: Vec2f,
    /// Kinda like angular momentum, except more special-casey.
    /// TODO Might wanna revisit when i have proper physics.
    pub turn_rate: f64,
}

#[must_use]
pub fn spawn_guided_missile(cvars: &Cvars, pos: Vec2f, angle: f64) -> GuidedMissile {
    // example of GM pasing through wall:
    // pos: Vec2f::new(640.0, 640.0),
    // vel: Vec2f::new(0.3, 0.2),

    GuidedMissile {
        pos,
        vel: Vec2f::new(cvars.g_guided_missile_speed_initial, 0.0).rotated_z(angle),
        turn_rate: 0.0,
    }
}

#[derive(Debug, Clone)]
pub struct Tank {
    pub pos: Vec2f,
    pub vel: Vec2f,
    pub angle: f64,
    pub angular_momentum: f64,
}

#[must_use]
pub fn spawn_tank(pos: Vec2f, angle: f64) -> Tank {
    Tank {
        pos,
        vel: Vec2f::zero(),
        angle,
        angular_momentum: 0.0,
    }
}

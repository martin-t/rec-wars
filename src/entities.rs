use rand::prelude::*;

use crate::data::{Map, Vec2f};
use crate::log;

#[derive(Debug, Clone)]
pub struct GuidedMissile {
    pub pos: Vec2f,
    pub vel: Vec2f,
    pub turn_rate: f64,
}

#[must_use]
pub fn spawn_guided_missile(rng: &mut SmallRng, map: &Map) -> GuidedMissile {
    // example of GM pasing through wall:
    // pos: Vec2f::new(640.0, 640.0),
    // vel: Vec2f::new(0.3, 0.2),

    let r = rng.gen_range(0, map.spawns().len());
    let spawn_index = map.spawns()[r];
    let spawn_pos = map.tile_center(spawn_index);
    let spawn_angle = map[spawn_index].rotation;

    log!("spawn: {}", spawn_index);

    GuidedMissile {
        pos: spawn_pos,
        vel: Vec2f::new(1.0, 0.0).rotated_z(spawn_angle),
        turn_rate: 0.0,
    }
}

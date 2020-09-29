//! The S in ECS.
//!
//! Not using legion's #[system] macro because:
//!
//! - Legion wants to own resources and state (cvars, map, RNG, ...).
//!   Both #[resource] and #[state] require the data to be passed by value (into Resources or the *_system() functions).
//!   There's no way to have them stored somewhere else and pass them as reference into the systems.
//!   This means I'd have to move everything into the ECS, which in turn would make even resources and state duck-typed
//!   when accessing them outside systems. Cvars are even worse because those have to be owned by JS.
//! - WASM currently only uses 1 thread anyway so no perf benefit from parallelism.
//! - https://github.com/amethyst/legion/issues/199 - I'd have to to split Pos
//!   into separate components for vehicles and projectiles to be able to do collision detection.

use legion::{query::IntoQuery, World};

use crate::{
    components::Bfg, components::Pos, components::Vel, cvars::Cvars, game_state::Explosion,
    game_state::GameState, map::Map,
};

pub(crate) fn bfg(cvars: &Cvars, world: &mut World, map: &Map, gs: &mut GameState) {
    let mut to_remove = Vec::new();

    let mut query = <(legion::Entity, &Bfg, &mut Pos, &Vel)>::query();
    for (&entity, _, pos, vel) in query.iter_mut(world) {
        pos.0 += vel.0 * gs.dt;

        if map.collision(pos.0) {
            gs.explosions.push(Explosion::new(
                pos.0,
                cvars.g_bfg_explosion_scale,
                gs.frame_time,
                true,
            ));
            to_remove.push(entity);
        }
    }

    for entity in to_remove {
        world.remove(entity);
    }
}

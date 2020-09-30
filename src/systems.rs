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

use legion::{query::IntoQuery, Entity, World};

use crate::{
    components::{Bfg, Destroyed, Owner, Pos, Vel},
    cvars::Cvars,
    entities,
    game_state::{Explosion, GameState},
    map::Map,
};

pub(crate) fn bfg(cvars: &Cvars, world: &mut World, map: &Map, gs: &mut GameState) {
    let vehicles = entities::all_vehicles(world);
    let mut to_remove = Vec::new();
    let mut to_kill = Vec::new();

    let mut query = <(Entity, &Bfg, &mut Pos, &Vel, &Owner)>::query();
    for (&proj_id, _, proj_pos, proj_vel, proj_owner) in query.iter_mut(world) {
        proj_pos.0 += proj_vel.0 * gs.dt;

        if map.collision(proj_pos.0) {
            gs.explosions.push(Explosion::new(
                proj_pos.0,
                cvars.g_bfg_explosion_scale,
                gs.frame_time,
                true,
            ));
            to_remove.push(proj_id);
        }

        for &(veh_id, veh_destroyed, veh_pos, _veh_angle, _veh_hitbox) in &vehicles {
            if !veh_destroyed.0
                && veh_id != proj_owner.0
                && (proj_pos.0 - veh_pos.0).magnitude_squared() <= 24.0 * 24.0
            {
                to_remove.push(proj_id);

                // Projectile explosion above vehicle explosion because it looks better.
                gs.explosions
                    .push(Explosion::new(veh_pos.0, 1.0, gs.frame_time, false));
                gs.explosions
                    .push(Explosion::new(proj_pos.0, 1.0, gs.frame_time, true));

                to_kill.push(veh_id);

                break;
            }
        }
    }

    for entity in to_remove {
        world.remove(entity);
    }

    for veh_id in to_kill {
        let mut entry = world.entry(veh_id).unwrap();
        let destroyed = entry.get_component_mut::<Destroyed>().unwrap();
        destroyed.0 = true;
    }
}

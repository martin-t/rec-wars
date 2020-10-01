//! The S in ECS.
//!
//! Not using legion's #[system] macro because:
//! - Legion wants to own resources and state (cvars, map, RNG, ...).
//!   Both #[resource] and #[state] require the data to be passed by value (into Resources or the *_system() functions).
//!   There's no way to have them stored somewhere else and pass them as reference into the systems.
//!   This means I'd have to move everything into the ECS, which in turn would make even resources and state duck-typed
//!   when accessing them outside systems. Cvars are even worse because those have to be owned by JS.
//! - WASM currently only uses 1 thread anyway so no perf benefit from parallelism.
//! - https://github.com/amethyst/legion/issues/199 - I'd have to to split Pos
//!   into separate components for vehicles and projectiles to be able to do collision detection.
//! - Simple functions like these can return data to be passed to other systems.

use legion::{query::IntoQuery, Entity, World};

use crate::{
    components::{Destroyed, Owner, Pos, Time, Vel, Weapon},
    cvars::Cvars,
    entities,
    game_state::{Explosion, GameState},
    map::Map,
};

pub(crate) fn projectiles(cvars: &Cvars, world: &mut World, gs: &mut GameState, map: &Map) {
    let vehicles = entities::all_vehicles(world);
    let mut to_remove = Vec::new();
    let mut to_kill = Vec::new();

    let mut query = <(Entity, &Weapon, &mut Pos, &Vel, &Owner)>::query();
    for (&proj_id, &proj_weap, proj_pos, proj_vel, proj_owner) in query.iter_mut(world) {
        proj_pos.0 += proj_vel.0 * gs.dt;

        if proj_weap == Weapon::Cb {
            continue;
        }

        if map.collision(proj_pos.0) {
            gs.explosions.push(Explosion::new(
                proj_pos.0,
                cvars.g_weapon_explosion_scale(proj_weap), // TODO MG, also below
                gs.frame_time,
                proj_weap == Weapon::Bfg,
            ));
            to_remove.push(proj_id);
            continue;
        }

        for &(veh_id, veh_destroyed, veh_pos, _veh_angle, _veh_hitbox) in &vehicles {
            if !veh_destroyed.0
                && veh_id != proj_owner.0
                && (proj_pos.0 - veh_pos.0).magnitude_squared() <= 24.0 * 24.0
            {
                to_remove.push(proj_id);

                // Vehicle explosion first to it's below projectile explosion because it looks better.
                gs.explosions
                    .push(Explosion::new(veh_pos.0, 1.0, gs.frame_time, false));
                gs.explosions.push(Explosion::new(
                    proj_pos.0,
                    cvars.g_weapon_explosion_scale(proj_weap),
                    gs.frame_time,
                    proj_weap == Weapon::Bfg,
                ));

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

/// Right now, CBs are the only timed projectiles, long term, might wanna add timeouts to more
/// to avoid too many entities on huge maps..
pub(crate) fn projectiles_timeout(cvars: &Cvars, world: &mut World, gs: &mut GameState) {
    let mut to_remove = Vec::new();

    let mut query = <(Entity, &Weapon, &mut Pos, &Time)>::query();
    for (&entity, &weap, pos, time) in query.iter_mut(world) {
        if gs.frame_time > time.0 {
            gs.explosions.push(Explosion::new(
                pos.0,
                cvars.g_weapon_explosion_scale(weap),
                time.0,
                weap == Weapon::Bfg,
            ));
            to_remove.push(entity);
        }
    }

    for entity in to_remove {
        world.remove(entity);
    }
}

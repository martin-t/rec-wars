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

use entities::{Ammo, GuidedMissile, Vehicle};
use legion::{query::IntoQuery, systems::CommandBuffer, Entity, World};
use rand::Rng;
use rand_distr::StandardNormal;

use crate::{
    components::{Angle, Destroyed, Owner, Pos, Time, VehicleType, Vel, Weapon},
    cvars::Cvars,
    cvars::Hardpoint,
    entities,
    game_state::{ControlledEntity, Explosion, GameState},
    map::F64Ext,
    map::Map,
    map::Vec2f,
};

pub(crate) fn turrets(cvars: &Cvars, world: &mut World, gs: &mut GameState) {
    let cur_weap = gs.cur_weapon;

    let mut query = <(&mut Vehicle,)>::query();
    for (vehicle,) in query.iter_mut(world) {
        // Turret turning
        if gs.input.turret_left {
            vehicle.turret_angle -= cvars.g_turret_turn_speed * gs.dt;
        }
        if gs.input.turret_right {
            vehicle.turret_angle += cvars.g_turret_turn_speed * gs.dt;
        }

        // Reloading
        let ammo = &mut vehicle.ammos[cur_weap as usize];
        if let Ammo::Reloading(_, end) = ammo {
            if gs.frame_time >= *end {
                *ammo = Ammo::Loaded(gs.frame_time, cvars.g_weapon_reload_ammo(cur_weap));
            }
        }
    }
}

pub(crate) fn shooting(cvars: &Cvars, world: &mut World, gs: &mut GameState, map: &Map) {
    let mut cmds = CommandBuffer::new(world);
    let mut query = <(
        Entity,
        &mut Vehicle,
        &VehicleType,
        &Destroyed,
        &Pos,
        &Vel,
        &Angle,
    )>::query();
    for (&veh_id, vehicle, veh_type, veh_destroyed, veh_pos, veh_vel, veh_angle) in
        query.iter_mut(world)
    {
        if veh_destroyed.0 || !gs.input.fire {
            continue;
        }
        let ammo = &mut vehicle.ammos[gs.cur_weapon as usize];
        if let Ammo::Loaded(ready_time, count) = ammo {
            if gs.frame_time < *ready_time {
                continue;
            }

            *ready_time = gs.frame_time + cvars.g_weapon_refire(gs.cur_weapon);
            *count -= 1;
            if *count == 0 {
                let reload_time = cvars.g_weapon_reload_time(gs.cur_weapon);
                *ammo = Ammo::Reloading(gs.frame_time, gs.frame_time + reload_time);
            }

            let (hardpoint, weapon_offset) = cvars.g_hardpoint(*veh_type, gs.cur_weapon);
            let (shot_angle, shot_origin);
            match hardpoint {
                Hardpoint::Chassis => {
                    shot_angle = veh_angle.0;
                    shot_origin = veh_pos.0 + weapon_offset.rotated_z(shot_angle);
                }
                Hardpoint::Turret => {
                    shot_angle = veh_angle.0 + vehicle.turret_angle;
                    let turret_offset = cvars.g_vehicle_turret_offset_chassis(*veh_type);
                    shot_origin = veh_pos.0
                        + turret_offset.rotated_z(veh_angle.0)
                        + weapon_offset.rotated_z(shot_angle);
                }
            }
            let pos = Pos(shot_origin);
            let owner = Owner(veh_id);
            match gs.cur_weapon {
                Weapon::Mg => {
                    let r: f64 = gs.rng.sample(StandardNormal);
                    let spread = cvars.g_machine_gun_angle_spread * r;
                    // Using spread as y would mean the resulting spread depends on speed
                    // so it's better to use spread on angle.
                    let shot_vel = Vec2f::new(cvars.g_machine_gun_speed, 0.0)
                        .rotated_z(shot_angle + spread)
                        + cvars.g_machine_gun_vehicle_velocity_factor * veh_vel.0;
                    let vel = Vel(shot_vel);
                    cmds.push((Weapon::Mg, pos, vel, owner));
                }
                Weapon::Rail => {
                    let dir = shot_angle.to_vec2f();
                    let end = shot_origin + dir * 100_000.0;
                    let hit = map.collision_between(shot_origin, end);
                    if let Some(hit) = hit {
                        gs.railguns.push((shot_origin, hit));
                    }
                }
                Weapon::Cb => {
                    for _ in 0..cvars.g_cluster_bomb_count {
                        let speed = cvars.g_cluster_bomb_speed;
                        let spread_forward;
                        let spread_sideways;
                        if cvars.g_cluster_bomb_speed_spread_gaussian {
                            // Broken type inference (works with rand crate but distributions are deprecated).
                            let r: f64 = gs.rng.sample(StandardNormal);
                            spread_forward = cvars.g_cluster_bomb_speed_spread_forward * r;
                            let r: f64 = gs.rng.sample(StandardNormal);
                            spread_sideways = cvars.g_cluster_bomb_speed_spread_sideways * r;
                        } else {
                            let r = gs.rng.gen_range(-1.5, 1.5);
                            spread_forward = cvars.g_cluster_bomb_speed_spread_forward * r;
                            let r = gs.rng.gen_range(-1.5, 1.5);
                            spread_sideways = cvars.g_cluster_bomb_speed_spread_sideways * r;
                        }
                        let shot_vel = Vec2f::new(speed + spread_forward, spread_sideways)
                            .rotated_z(shot_angle)
                            + cvars.g_cluster_bomb_vehicle_velocity_factor * veh_vel.0;
                        let vel = Vel(shot_vel);
                        let time = gs.frame_time
                            + cvars.g_cluster_bomb_time
                            + gs.rng.gen_range(-1.0, 1.0) * cvars.g_cluster_bomb_time_spread;
                        let time = Time(time);
                        cmds.push((Weapon::Cb, pos, vel, time, owner));
                    }
                }
                Weapon::Rockets => {
                    let shot_vel = Vec2f::new(cvars.g_rockets_speed, 0.0).rotated_z(shot_angle)
                        + cvars.g_rockets_vehicle_velocity_factor * veh_vel.0;
                    let vel = Vel(shot_vel);
                    cmds.push((Weapon::Rockets, pos, vel, owner));
                }
                Weapon::Hm => {
                    let shot_vel = Vec2f::new(cvars.g_homing_missile_speed_initial, 0.0)
                        .rotated_z(shot_angle)
                        + cvars.g_homing_missile_vehicle_velocity_factor * veh_vel.0;
                    let vel = Vel(shot_vel);
                    cmds.push((Weapon::Hm, pos, vel, owner));
                }
                Weapon::Gm => {
                    gs.gm = GuidedMissile::spawn(cvars, shot_origin, shot_angle);
                    gs.ce = ControlledEntity::GuidedMissile;
                }
                Weapon::Bfg => {
                    let shot_vel = Vec2f::new(cvars.g_bfg_speed, 0.0).rotated_z(shot_angle)
                        + cvars.g_bfg_vehicle_velocity_factor * veh_vel.0;
                    let vel = Vel(shot_vel);
                    cmds.push((Weapon::Bfg, pos, vel, owner));
                }
            }
        }
    }
    cmds.flush(world);
}

pub(crate) fn projectiles(cvars: &Cvars, world: &mut World, gs: &mut GameState, map: &Map) {
    let vehicles = entities::all_vehicles(world);
    let mut to_remove = Vec::new();
    let mut to_kill = Vec::new();

    let mut query = <(Entity, &Weapon, &mut Pos, &Vel, &Owner)>::query();
    for (&proj_id, &proj_weap, proj_pos, proj_vel, proj_owner) in query.iter_mut(world) {
        let new_pos = proj_pos.0 + proj_vel.0 * gs.dt;

        if proj_weap == Weapon::Cb {
            proj_pos.0 = new_pos;
            continue;
        }

        let collision = map.collision_between(proj_pos.0, new_pos);
        if let Some(col_pos) = collision {
            if let Some(expl_scale) = cvars.g_weapon_explosion_scale(proj_weap) {
                gs.explosions.push(Explosion::new(
                    col_pos,
                    expl_scale,
                    gs.frame_time,
                    proj_weap == Weapon::Bfg,
                ));
            }

            to_remove.push(proj_id);
            continue;
        }

        proj_pos.0 = new_pos;

        for &(veh_id, veh_destroyed, veh_pos, _veh_angle, _veh_hitbox) in &vehicles {
            if !veh_destroyed.0
                && veh_id != proj_owner.0
                && (proj_pos.0 - veh_pos.0).magnitude_squared() <= 24.0 * 24.0
            {
                to_remove.push(proj_id);

                // Vehicle explosion first to it's below projectile explosion because it looks better.
                gs.explosions
                    .push(Explosion::new(veh_pos.0, 1.0, gs.frame_time, false));
                if let Some(expl_scale) = cvars.g_weapon_explosion_scale(proj_weap) {
                    gs.explosions.push(Explosion::new(
                        proj_pos.0,
                        expl_scale,
                        gs.frame_time,
                        proj_weap == Weapon::Bfg,
                    ));
                }

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
            if let Some(expl_scale) = cvars.g_weapon_explosion_scale(weap) {
                gs.explosions.push(Explosion::new(
                    pos.0,
                    expl_scale,
                    time.0,
                    weap == Weapon::Bfg,
                ));
            }
            to_remove.push(entity);
        }
    }

    for entity in to_remove {
        world.remove(entity);
    }
}

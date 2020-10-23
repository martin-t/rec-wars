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

use std::f64::consts::PI;

use legion::{query::IntoQuery, systems::CommandBuffer, Entity, World};
use rand::Rng;
use rand_distr::StandardNormal;
use vek::Clamp;

use crate::{
    components::Player,
    components::{
        Ammo, Angle, Bfg, Cb, GuidedMissile, Hitbox, Mg, Owner, Pos, Time, TurnRate, Vehicle, Vel,
        Weapon, WEAPS_CNT,
    },
    cvars::Cvars,
    cvars::Hardpoint,
    cvars::MovementStats,
    game_state::{Explosion, GameState, Input, EMPTY_INPUT},
    map::F64Ext,
    map::Map,
    map::Vec2f,
    map::VecExt,
};

pub(crate) fn self_destruct(cvars: &Cvars, world: &mut World, gs: &mut GameState) {
    let mut query = <(&mut Vehicle, &mut Pos, &Input)>::query();
    for (vehicle, veh_pos, input) in query.iter_mut(world) {
        if input.self_destruct && !vehicle.destroyed() {
            vehicle.hp_fraction = 0.0;
            gs.explosions.push(Explosion::new(
                veh_pos.0,
                cvars.g_self_destruct_explosion1_scale,
                gs.frame_time,
                false,
            ));
            gs.explosions.push(Explosion::new(
                veh_pos.0,
                cvars.g_self_destruct_explosion2_scale,
                gs.frame_time,
                false,
            ));
        }
    }
}

pub(crate) fn vehicle_movement(cvars: &Cvars, world: &mut World, gs: &GameState, map: &Map) {
    let mut query = <(
        &Vehicle,
        &mut Pos,
        &mut Vel,
        &mut Angle,
        &mut TurnRate,
        &Hitbox,
        &Input,
    )>::query();
    for (vehicle, pos, vel, angle, turn_rate, hitbox, input) in query.iter_mut(world) {
        let stats = cvars.g_vehicle_movement_stats(vehicle.veh_type);

        let new_angle = turning(&stats, vel, angle, turn_rate, input, gs.dt);

        if hitbox
            .corners(pos.0, new_angle)
            .iter()
            .any(|&corner| map.collision(corner))
        {
            turn_rate.0 *= -0.5;
        } else {
            angle.0 = new_angle;
        }

        accel_decel(&stats, vel, angle, input, gs.dt);

        // Moving
        let new_pos = pos.0 + vel.0 * gs.dt;
        if hitbox
            .corners(new_pos, angle.0)
            .iter()
            .any(|&corner| map.collision(corner))
        {
            vel.0 *= -0.5;
        } else {
            pos.0 = new_pos;
        }
    }
}

fn turning(
    stats: &MovementStats,
    vel: &mut Vel,
    angle: &Angle,
    turn_rate: &mut TurnRate,
    input: &Input,
    dt: f64,
) -> f64 {
    let tr_change = input.right_left() * stats.turn_rate_increase * dt;
    turn_rate.0 += tr_change;

    // Friction's constant component - always the same no matter the speed
    let tr_fric_const = stats.turn_rate_friction_const * dt;
    if turn_rate.0 >= 0.0 {
        turn_rate.0 = (turn_rate.0 - tr_fric_const).max(0.0);
    } else {
        turn_rate.0 = (turn_rate.0 + tr_fric_const).min(0.0);
    }

    // Friction's linear component - increases with speed
    let tr_new = turn_rate.0 * (1.0 - stats.turn_rate_friction_linear).powf(dt);
    turn_rate.0 = tr_new.clamped(-stats.turn_rate_max, stats.turn_rate_max);

    // A dirty hack to approximate car steering (i.e. no turning when still, reversed when moving backwards).
    let steering_coef = if stats.steering_car > 0.0 {
        let sign = angle.0.to_vec2f().dot(vel.0).signum();
        // Steering when below this speed is less effective.
        let steering_speed = vel
            .0
            .magnitude()
            .clamped(-stats.steering_car, stats.steering_car);
        steering_speed * sign / stats.steering_car
    } else {
        1.0
    };

    // Turning - part of vel gets rotated to simulate steering
    let turn = turn_rate.0 * dt * steering_coef;
    let vel_rotation = turn * stats.turn_effectiveness;
    vel.0.rotate_z(vel_rotation);

    // Normalize to 0..=360 deg
    (angle.0 + turn).rem_euclid(2.0 * PI)
}

fn accel_decel(stats: &MovementStats, vel: &mut Vel, angle: &mut Angle, input: &Input, dt: f64) {
    let vel_change = (input.up() * stats.accel_forward - input.down() * stats.accel_backward) * dt;
    vel.0 += angle.0.to_vec2f() * vel_change;

    // Friction's constant component - always the same no matter the speed
    let vel_fric_const = stats.friction_const * dt;
    let vel_norm = vel.0.try_normalized().unwrap_or_default();
    vel.0 -= (vel_fric_const).min(vel.0.magnitude()) * vel_norm;

    // Friction's linear component - increases with speed
    vel.0 *= (1.0 - stats.friction_linear).powf(dt);
    if vel.0.magnitude_squared() > stats.speed_max.powi(2) {
        vel.0 = vel_norm * stats.speed_max;
    }
}

pub(crate) fn vehicle_logic(
    cvars: &Cvars,
    world: &mut World,
    gs: &mut GameState,
    gs_prev: &GameState,
) {
    let mut query = <(&mut Vehicle, &Input)>::query();
    for (vehicle, input) in query.iter_mut(world) {
        // Change weapon
        if input.prev_weapon && !gs_prev.input.prev_weapon {
            let prev = (vehicle.cur_weapon as u8 + WEAPS_CNT - 1) % WEAPS_CNT;
            vehicle.cur_weapon = Weapon::n(prev).unwrap();
        }
        if input.next_weapon && !gs_prev.input.next_weapon {
            let next = (vehicle.cur_weapon as u8 + 1) % WEAPS_CNT;
            vehicle.cur_weapon = Weapon::n(next).unwrap();
        }

        // Turret turning
        if input.turret_left {
            vehicle.turret_angle -= cvars.g_turret_turn_speed * gs.dt;
        }
        if input.turret_right {
            vehicle.turret_angle += cvars.g_turret_turn_speed * gs.dt;
        }

        // Reloading
        let ammo = &mut vehicle.ammos[vehicle.cur_weapon as usize];
        if let Ammo::Reloading(_, end) = ammo {
            if gs.frame_time >= *end {
                *ammo = Ammo::Loaded(
                    gs.frame_time,
                    cvars.g_weapon_reload_ammo(vehicle.cur_weapon),
                );
            }
        }
    }
}

pub(crate) fn shooting(cvars: &Cvars, world: &mut World, gs: &mut GameState, map: &Map) {
    let mut cmds = CommandBuffer::new(world);
    let mut query = <(&mut Vehicle, &Pos, &Vel, &Angle, &Input, &Owner)>::query();
    for (vehicle, veh_pos, veh_vel, veh_angle, input, &owner) in query.iter_mut(world) {
        if vehicle.destroyed() || !input.fire {
            continue;
        }
        let ammo = &mut vehicle.ammos[vehicle.cur_weapon as usize];
        if let Ammo::Loaded(ready_time, count) = ammo {
            if gs.frame_time < *ready_time {
                continue;
            }

            *ready_time = gs.frame_time + cvars.g_weapon_refire(vehicle.cur_weapon);
            *count -= 1;
            if *count == 0 {
                let reload_time = cvars.g_weapon_reload_time(vehicle.cur_weapon);
                *ammo = Ammo::Reloading(gs.frame_time, gs.frame_time + reload_time);
            }

            let (hardpoint, weapon_offset) =
                cvars.g_hardpoint(vehicle.veh_type, vehicle.cur_weapon);
            let (shot_angle, shot_origin);
            match hardpoint {
                Hardpoint::Chassis => {
                    shot_angle = veh_angle.0;
                    shot_origin = veh_pos.0 + weapon_offset.rotated_z(shot_angle);
                }
                Hardpoint::Turret => {
                    shot_angle = veh_angle.0 + vehicle.turret_angle;
                    let turret_offset = cvars.g_vehicle_turret_offset_chassis(vehicle.veh_type);
                    shot_origin = veh_pos.0
                        + turret_offset.rotated_z(veh_angle.0)
                        + weapon_offset.rotated_z(shot_angle);
                }
            }
            let pos = Pos(shot_origin);
            match vehicle.cur_weapon {
                Weapon::Mg => {
                    let r: f64 = gs.rng.sample(StandardNormal);
                    let spread = cvars.g_machine_gun_angle_spread * r;
                    // Using spread as y would mean the resulting spread depends on speed
                    // so it's better to use spread on angle.
                    let shot_vel = Vec2f::new(cvars.g_machine_gun_speed, 0.0)
                        .rotated_z(shot_angle + spread)
                        + cvars.g_machine_gun_vehicle_velocity_factor * veh_vel.0;
                    let vel = Vel(shot_vel);
                    cmds.push((Weapon::Mg, Mg, pos, vel, owner));
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
                        cmds.push((Weapon::Cb, Cb, pos, vel, time, owner));
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
                    let gm = GuidedMissile;
                    let shot_vel = Vec2f::new(cvars.g_guided_missile_speed_initial, 0.0)
                        .rotated_z(shot_angle)
                        + cvars.g_guided_missile_vehicle_velocity_factor * veh_vel.0;
                    let vel = Vel(shot_vel);
                    let angle = Angle(vel.0.to_angle());
                    let tr = TurnRate(0.0);
                    let player = owner.0;
                    let gm_entity =
                        cmds.push((Weapon::Gm, gm, pos, vel, angle, tr, owner, EMPTY_INPUT));
                    cmds.exec_mut(move |legion| {
                        legion
                            .entry(player)
                            .unwrap()
                            .get_component_mut::<Player>()
                            .unwrap()
                            .guided_missile = Some(gm_entity);
                    });
                }
                Weapon::Bfg => {
                    let shot_vel = Vec2f::new(cvars.g_bfg_speed, 0.0).rotated_z(shot_angle)
                        + cvars.g_bfg_vehicle_velocity_factor * veh_vel.0;
                    let vel = Vel(shot_vel);
                    cmds.push((Weapon::Bfg, Bfg, pos, vel, owner));
                }
            }
        }
    }
    cmds.flush(world);
}

pub(crate) fn gm_turning(cvars: &Cvars, world: &mut World, gs: &GameState) {
    let mut query = <(&GuidedMissile, &mut Vel, &mut Angle, &mut TurnRate, &Input)>::query();
    for (_, vel, angle, turn_rate, input) in query.iter_mut(world) {
        let stats = cvars.g_weapon_movement_stats();

        angle.0 = turning(&stats, vel, angle, turn_rate, input, gs.dt);

        accel_decel(&stats, vel, angle, input, gs.dt);
    }
}

pub(crate) fn projectiles(cvars: &Cvars, world: &mut World, gs: &mut GameState, map: &Map) {
    let mut query_vehicles = <(Entity, &Vehicle, &Pos, &Angle, &Hitbox, &Owner)>::query();
    let vehicles: Vec<(Entity, _, _, _, _)> = query_vehicles
        .iter(world)
        .filter_map(|(&veh_id, vehicle, &pos, &angle, &hitbox, &owner)| {
            if !vehicle.destroyed() {
                Some((veh_id, pos, angle, hitbox, owner))
            } else {
                None
            }
        })
        .collect();

    let mut cmds = CommandBuffer::new(world);

    let mut query_projectiles = <(Entity, &Weapon, &mut Pos, &Vel, &Owner)>::query();
    let (mut world_projectiles, mut world_rest) = world.split_for_query(&query_projectiles);
    for (&proj_id, &proj_weap, proj_pos, proj_vel, proj_owner) in
        query_projectiles.iter_mut(&mut world_projectiles)
    {
        let new_pos = proj_pos.0 + proj_vel.0 * gs.dt;

        if proj_weap == Weapon::Cb {
            proj_pos.0 = new_pos;
            continue;
        }

        let collision = map.collision_between(proj_pos.0, new_pos);
        if let Some(hit_pos) = collision {
            remove_projectile(
                cvars,
                gs,
                &mut cmds,
                proj_id,
                proj_weap,
                proj_owner.0,
                hit_pos,
            );
            continue;
        }

        proj_pos.0 = new_pos;

        for (veh_id, veh_pos, _veh_angle, _veh_hitbox, veh_owner) in &vehicles {
            let veh_id = *veh_id;
            if veh_owner != proj_owner {
                let dist2 = (proj_pos.0 - veh_pos.0).magnitude_squared();
                // TODO proper hitbox
                if dist2 <= 24.0 * 24.0 {
                    let mut query_veh = <(&mut Vehicle,)>::query();
                    let (vehicle,) = query_veh.get_mut(&mut world_rest, veh_id).unwrap();
                    let dmg = cvars.g_weapon_damage(proj_weap);
                    vehicle.damage(cvars, dmg);
                    // Vehicle explosion first so it's below projectile explosion because it looks better.
                    if vehicle.destroyed() {
                        gs.explosions
                            .push(Explosion::new(veh_pos.0, 1.0, gs.frame_time, false));
                    }
                    remove_projectile(
                        cvars,
                        gs,
                        &mut cmds,
                        proj_id,
                        proj_weap,
                        proj_owner.0,
                        proj_pos.0,
                    );
                    break;
                } else if proj_weap == Weapon::Bfg
                    && dist2 <= cvars.g_bfg_beam_range * cvars.g_bfg_beam_range
                    && map.collision_between(proj_pos.0, veh_pos.0).is_none()
                {
                    let mut query_veh = <(&mut Vehicle,)>::query();
                    let (vehicle,) = query_veh.get_mut(&mut world_rest, veh_id).unwrap();
                    let dmg = cvars.g_bfg_beam_damage_per_sec * gs.dt;
                    vehicle.damage(cvars, dmg);
                    if vehicle.destroyed() {
                        gs.explosions
                            .push(Explosion::new(veh_pos.0, 1.0, gs.frame_time, false));
                    }
                    gs.bfg_beams.push((proj_pos.0, veh_pos.0));
                }
            }
        }
    }

    cmds.flush(world);
}

/// Right now, CBs are the only timed projectiles, long term, might wanna add timeouts to more
/// to avoid too many entities on huge maps..
pub(crate) fn projectiles_timeout(cvars: &Cvars, world: &mut World, gs: &mut GameState) {
    let mut cmds = CommandBuffer::new(world);

    let mut query = <(Entity, &Weapon, &Pos, &Time, &Owner)>::query();
    for (&entity, &weap, pos, time, owner) in query.iter(world) {
        if gs.frame_time > time.0 {
            remove_projectile(cvars, gs, &mut cmds, entity, weap, owner.0, pos.0);
        }
    }

    cmds.flush(world);
}

fn remove_projectile(
    cvars: &Cvars,
    gs: &mut GameState,
    cmds: &mut CommandBuffer,
    proj: Entity,
    proj_weap: Weapon,
    proj_owner: Entity,
    hit_pos: Vec2f,
) {
    if let Some(expl_scale) = cvars.g_weapon_explosion_scale(proj_weap) {
        gs.explosions.push(Explosion::new(
            hit_pos,
            expl_scale,
            gs.frame_time,
            proj_weap == Weapon::Bfg,
        ));
    }
    if proj_weap == Weapon::Gm {
        cmds.exec_mut(move |world| {
            world
                .entry(proj_owner)
                .unwrap()
                .get_component_mut::<Player>()
                .unwrap()
                .guided_missile = None;
        });
    }
    cmds.remove(proj);
}

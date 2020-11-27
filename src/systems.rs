//! The S in ECS
//!
//! We're using the ECS design pattern (decouple behavior from data),
//! just without the ECS data structure (we use generational arenas instead).
//! Most game behavior (code that changes state) goes here.

pub(crate) mod ai;
pub(crate) mod rendering;

use std::f64::consts::PI;

use rand::Rng;
use rand_distr::StandardNormal;
use thunderdome::Index;
use vek::{Clamp, LineSegment2, Wrap};

use crate::{
    cvars::{Cvars, Hardpoint, MovementStats},
    entities::{Ammo, Projectile, Vehicle, VehicleType, Weapon, WEAPS_CNT},
    game_state::ArenaExt,
    game_state::{Explosion, GameState, Input},
    map::{F64Ext, Map, Vec2f},
};

pub(crate) fn cleanup(cvars: &Cvars, gs: &mut GameState) {
    // Cleanup old entities
    gs.railguns.clear();
    gs.bfg_beams.clear();
    let frame_time = gs.frame_time; // borrowchk
    gs.explosions.retain(|explosion| {
        let progress = (frame_time - explosion.start_time) / cvars.r_explosion_duration;
        progress <= 1.0
    });
}

pub(crate) fn respawning(cvars: &Cvars, gs: &mut GameState, gs_prev: &GameState, map: &Map) {
    for player_handle in gs.players.iter_handles() {
        let player = &mut gs.players[player_handle];
        let input_prev = gs_prev.players[player_handle].input;
        if let Some(vehicle_handle) = player.vehicle {
            let destroyed = gs.vehicles[vehicle_handle].destroyed();

            if destroyed && player.input.fire && !input_prev.fire {
                gs.vehicles.remove(vehicle_handle).unwrap();

                spawn_vehicle(cvars, gs, map, player_handle, true);
            }
        } else if player.input.fire && !input_prev.fire {
            spawn_vehicle(cvars, gs, map, player_handle, true);
        }
    }
}

pub(crate) fn spawn_vehicle(
    cvars: &Cvars,
    gs: &mut GameState,
    map: &Map,
    player_handle: Index,
    use_spawns: bool,
) {
    let veh_type = VehicleType::n(gs.rng.gen_range(0, 3)).unwrap();
    let (spawn_pos, spawn_angle) = if use_spawns {
        map.random_spawn(&mut gs.rng)
    } else {
        let (pos, _angle) = map.random_nonwall(&mut gs.rng);
        // Most grass tiles have no rotation so everyone ends up facing right which looks bad.
        // Throw away their angle and use a random one.
        let angle = gs.rng.gen_range(0.0, 2.0 * PI);
        (pos, angle)
    };

    let vehicle_handle = gs.vehicles.insert(Vehicle::new(
        cvars,
        spawn_pos,
        spawn_angle,
        veh_type,
        gs.frame_time,
        player_handle,
    ));

    let player = &mut gs.players[player_handle];
    player.vehicle = Some(vehicle_handle);
}

pub(crate) fn self_destruct(cvars: &Cvars, gs: &mut GameState) {
    for vehicle_handle in gs.vehicles.iter_handles() {
        let vehicle = &gs.vehicles[vehicle_handle];
        let input = &gs.players[vehicle.owner].input;
        if !input.self_destruct || vehicle.destroyed() {
            continue;
        }

        // First the big explosion...
        gs.explosions.push(Explosion::new(
            vehicle.pos,
            cvars.g_self_destruct_explosion_scale,
            gs.frame_time,
            false,
        ));
        // ...then destroy the vehicle to create the small explosion on top.
        let attacker_handle = vehicle.owner;
        damage(cvars, gs, attacker_handle, vehicle_handle, f64::MAX)
    }
}

pub(crate) fn vehicle_movement(cvars: &Cvars, gs: &mut GameState, map: &Map) {
    for (_, vehicle) in gs.vehicles.iter_mut() {
        let stats = cvars.g_vehicle_movement_stats(vehicle.veh_type);

        // No movement after death or when guiding
        let input = if vehicle.destroyed() {
            Input::new()
        } else {
            let player = &gs.players[vehicle.owner];
            if player.guided_missile.is_some() {
                gs.players[vehicle.owner].input.vehicle_while_guiding()
            } else {
                gs.players[vehicle.owner].input
            }
        };
        let new_angle = turning(
            &stats,
            &mut vehicle.vel,
            &vehicle.angle,
            &mut vehicle.turn_rate,
            input,
            gs.dt,
        );

        if vehicle
            .hitbox
            .corners(vehicle.pos, new_angle)
            .iter()
            .any(|&corner| map.is_wall(corner))
        {
            vehicle.turn_rate *= -0.5;
        } else {
            vehicle.angle = new_angle;
        }

        accel_decel(&stats, &mut vehicle.vel, &mut vehicle.angle, input, gs.dt);

        let new_pos = vehicle.pos + vehicle.vel * gs.dt;
        if vehicle
            .hitbox
            .corners(new_pos, vehicle.angle)
            .iter()
            .any(|&corner| map.is_wall(corner))
        {
            vehicle.vel *= -0.5;
        } else {
            vehicle.pos = new_pos;
        }
    }
}

fn turning(
    stats: &MovementStats,
    vel: &mut Vec2f,
    angle: &f64,
    turn_rate: &mut f64,
    input: Input,
    dt: f64,
) -> f64 {
    let tr_change = input.right_left() * stats.turn_rate_increase * dt;
    *turn_rate += tr_change;

    // Friction's constant component - always the same no matter the speed
    let tr_fric_const = stats.turn_rate_friction_const * dt;
    if *turn_rate >= 0.0 {
        *turn_rate = (*turn_rate - tr_fric_const).max(0.0);
    } else {
        *turn_rate = (*turn_rate + tr_fric_const).min(0.0);
    }

    // Friction's linear component - increases with speed
    let tr_new = *turn_rate * (1.0 - stats.turn_rate_friction_linear).powf(dt);
    *turn_rate = tr_new.clamped(-stats.turn_rate_max, stats.turn_rate_max);

    // A dirty hack to approximate car steering (i.e. no turning when still, reversed when moving backwards).
    let steering_coef = if stats.steering_car > 0.0 {
        let sign = angle.to_vec2f().dot(*vel).signum();
        // Steering when below this speed is less effective.
        let steering_speed = vel
            .magnitude()
            .clamped(-stats.steering_car, stats.steering_car);
        steering_speed * sign / stats.steering_car
    } else {
        1.0
    };

    // Turning - part of vel gets rotated to simulate steering
    let turn = *turn_rate * dt * steering_coef;
    let vel_rotation = turn * stats.turn_effectiveness;
    vel.rotate_z(vel_rotation);

    // Normalize to 0..=360 deg
    (angle + turn).rem_euclid(2.0 * PI)
}

fn accel_decel(stats: &MovementStats, vel: &mut Vec2f, angle: &mut f64, input: Input, dt: f64) {
    let vel_change = (input.up() * stats.accel_forward - input.down() * stats.accel_backward) * dt;
    *vel += angle.to_vec2f() * vel_change;

    // Friction's constant component - always the same no matter the speed
    let vel_fric_const = stats.friction_const * dt;
    let vel_norm = vel.try_normalized().unwrap_or_default();
    *vel -= (vel_fric_const).min(vel.magnitude()) * vel_norm;

    // Friction's linear component - increases with speed
    *vel *= (1.0 - stats.friction_linear).powf(dt);
    if vel.magnitude_squared() > stats.speed_max.powi(2) {
        *vel = vel_norm * stats.speed_max;
    }
}

pub(crate) fn player_logic(gs: &mut GameState, gs_prev: &GameState) {
    for (player_index, player) in gs.players.iter_mut() {
        let input_prev = if let Some(player_prev) = gs_prev.players.get(player_index) {
            player_prev.input
        } else {
            // The player not might have been connected last frame.
            Input::new()
        };

        // Change weapon
        if player.input.prev_weapon && !input_prev.prev_weapon {
            let prev = (player.cur_weapon as u8 + WEAPS_CNT - 1) % WEAPS_CNT;
            player.cur_weapon = Weapon::n(prev).unwrap();
        }
        if player.input.next_weapon && !input_prev.next_weapon {
            let next = (player.cur_weapon as u8 + 1) % WEAPS_CNT;
            player.cur_weapon = Weapon::n(next).unwrap();
        }
    }
}

pub(crate) fn vehicle_logic(cvars: &Cvars, gs: &mut GameState, gs_prev: &GameState) {
    for (_, vehicle) in gs.vehicles.iter_mut() {
        // This should run even while dead, otherwise the ammo indicator will be buggy.
        // Original RW also reloaded while dead.

        let player = &gs.players[vehicle.owner];
        let input = &player.input;
        let input_prev = if let Some(player_prev) = gs_prev.players.get(vehicle.owner) {
            player_prev.input
        } else {
            // The player not might have been connected last frame.
            Input::new()
        };

        // Turret turning
        if input.turret_left && !input_prev.turret_left {
            vehicle.turret_angle_wanted -= cvars.g_turret_turn_step_angle_deg.to_radians();
        }
        if input.turret_right && !input_prev.turret_right {
            vehicle.turret_angle_wanted += cvars.g_turret_turn_step_angle_deg.to_radians();
        }
        vehicle.turret_angle_wanted = vehicle.turret_angle_wanted.rem_euclid(2.0 * PI);

        let delta = vehicle
            .turret_angle_current
            .delta_angle(vehicle.turret_angle_wanted);
        let change = cvars.g_turret_turn_speed_deg.to_radians() * gs.dt * delta.signum();
        let change_clamped = change.clamped(-delta.abs(), delta.abs());
        vehicle.turret_angle_current += change_clamped;
        vehicle.turret_angle_current = vehicle.turret_angle_current.rem_euclid(2.0 * PI);

        // Reloading
        let ammo = &mut vehicle.ammos[player.cur_weapon as usize];
        if let Ammo::Reloading(_, end) = ammo {
            if gs.frame_time >= *end {
                *ammo = Ammo::Loaded(gs.frame_time, cvars.g_weapon_reload_ammo(player.cur_weapon));
            }
        }
    }
}

pub(crate) fn shooting(cvars: &Cvars, gs: &mut GameState) {
    for (_, vehicle) in gs.vehicles.iter_mut() {
        let player = &mut gs.players[vehicle.owner];
        // Note: vehicles can shoot while controlling a missile
        if vehicle.destroyed() || !player.input.fire {
            continue;
        }

        let ammo = &mut vehicle.ammos[player.cur_weapon as usize];
        if let Ammo::Loaded(ready_time, count) = ammo {
            if gs.frame_time < *ready_time {
                continue;
            }

            *ready_time = gs.frame_time + cvars.g_weapon_refire(player.cur_weapon);
            *count -= 1;
            if *count == 0 {
                let reload_time = cvars.g_weapon_reload_time(player.cur_weapon);
                *ammo = Ammo::Reloading(gs.frame_time, gs.frame_time + reload_time);
            }

            let (hardpoint, weapon_offset) = cvars.g_hardpoint(vehicle.veh_type, player.cur_weapon);
            let (shot_angle, shot_origin);
            match hardpoint {
                Hardpoint::Chassis => {
                    shot_angle = vehicle.angle;
                    shot_origin = vehicle.pos + weapon_offset.rotated_z(shot_angle);
                }
                Hardpoint::Turret => {
                    shot_angle = vehicle.angle + vehicle.turret_angle_current;
                    let turret_offset = cvars.g_vehicle_turret_offset_chassis(vehicle.veh_type);
                    shot_origin = vehicle.pos
                        + turret_offset.rotated_z(vehicle.angle)
                        + weapon_offset.rotated_z(shot_angle);
                }
            }

            // Some sane defaults to be overriden later
            let mut projectile = Projectile {
                weapon: Weapon::Mg,
                pos: shot_origin,
                vel: Vec2f::zero(),
                angle: shot_angle,
                turn_rate: 0.0,
                explode_time: f64::MAX,
                owner: vehicle.owner,
            };

            match player.cur_weapon {
                Weapon::Mg => {
                    let r: f64 = gs.rng.sample(StandardNormal);
                    let spread = cvars.g_machine_gun_angle_spread * r;
                    // Using spread as shot_vel.y would mean the resulting spread depends on speed
                    // so it's better to use spread on angle.
                    projectile.vel = Vec2f::new(cvars.g_machine_gun_speed, 0.0)
                        .rotated_z(shot_angle + spread)
                        + cvars.g_machine_gun_vehicle_velocity_factor * vehicle.vel;
                    gs.projectiles.insert(projectile);
                }
                Weapon::Rail => {
                    projectile.weapon = Weapon::Rail;
                    projectile.vel = Vec2f::new(cvars.g_railgun_speed, 0.0).rotated_z(shot_angle)
                        + cvars.g_railgun_vehicle_velocity_factor * vehicle.vel;
                    gs.projectiles.insert(projectile);
                }
                Weapon::Cb => {
                    projectile.weapon = Weapon::Cb;
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
                        projectile.vel = Vec2f::new(speed + spread_forward, spread_sideways)
                            .rotated_z(shot_angle)
                            + cvars.g_cluster_bomb_vehicle_velocity_factor * vehicle.vel;
                        projectile.explode_time = gs.frame_time
                            + cvars.g_cluster_bomb_time
                            + gs.rng.gen_range(-1.0, 1.0) * cvars.g_cluster_bomb_time_spread;
                        gs.projectiles.insert(projectile.clone());
                    }
                }
                Weapon::Rockets => {
                    projectile.weapon = Weapon::Rockets;
                    projectile.vel = Vec2f::new(cvars.g_rockets_speed, 0.0).rotated_z(shot_angle)
                        + cvars.g_rockets_vehicle_velocity_factor * vehicle.vel;
                    gs.projectiles.insert(projectile);
                }
                Weapon::Hm => {
                    projectile.weapon = Weapon::Hm;
                    projectile.vel = Vec2f::new(cvars.g_homing_missile_speed_initial, 0.0)
                        .rotated_z(shot_angle)
                        + cvars.g_homing_missile_vehicle_velocity_factor * vehicle.vel;
                    gs.projectiles.insert(projectile);
                }
                Weapon::Gm => {
                    projectile.weapon = Weapon::Gm;
                    projectile.vel = Vec2f::new(cvars.g_guided_missile_speed_initial, 0.0)
                        .rotated_z(shot_angle)
                        + cvars.g_guided_missile_vehicle_velocity_factor * vehicle.vel;
                    // TODO angle (maybe also HM)
                    let handle = gs.projectiles.insert(projectile);
                    player.guided_missile = Some(handle);
                }
                Weapon::Bfg => {
                    projectile.weapon = Weapon::Bfg;
                    projectile.vel = Vec2f::new(cvars.g_bfg_speed, 0.0).rotated_z(shot_angle)
                        + cvars.g_bfg_vehicle_velocity_factor * vehicle.vel;
                    gs.projectiles.insert(projectile);
                }
            }
        }
    }
}

/// The *guided* part of guided missile
pub(crate) fn gm_turning(cvars: &Cvars, gs: &mut GameState) {
    for (gm_handle, gm) in gs
        .projectiles
        .iter_mut()
        .filter(|(_, proj)| proj.weapon == Weapon::Gm)
    {
        let stats = cvars.g_weapon_movement_stats();
        let player = &gs.players[gm.owner];

        // Only allow guiding the most recently launched missile
        let input = if player.guided_missile == Some(gm_handle) {
            player.input.missile_while_guiding()
        } else {
            Input::new_up()
        };

        gm.angle = turning(
            &stats,
            &mut gm.vel,
            &gm.angle,
            &mut gm.turn_rate,
            input,
            gs.dt,
        );

        accel_decel(&stats, &mut gm.vel, &mut gm.angle, input, gs.dt);
    }
}

/// Projectile movement and collisions / hit detection.
/// Traces the projectile's path between positions to avoid passing through thin objects.
pub(crate) fn projectiles(cvars: &Cvars, gs: &mut GameState, map: &Map) {
    for proj_handle in gs.projectiles.iter_handles() {
        let projectile = &mut gs.projectiles[proj_handle];
        let max_new_pos = projectile.pos + projectile.vel * gs.dt;

        if projectile.weapon == Weapon::Cb {
            projectile.pos = max_new_pos;
            continue;
        }

        let maybe_collision = map.is_wall_trace(projectile.pos, max_new_pos);
        let new_pos = if let Some(hit_pos) = maybe_collision {
            hit_pos
        } else {
            max_new_pos
        };

        if cvars.d_tracing {
            dbg_line!(projectile.pos, new_pos, 0.5);
        }
        let step = LineSegment2 {
            start: projectile.pos,
            end: new_pos,
        };
        let step_dir = (new_pos - projectile.pos).normalized();

        projectile.pos = new_pos;

        for vehicle_handle in gs.vehicles.iter_handles() {
            // LATER immediately killing vehicles here means 2 players can't share a kill
            let vehicle = &mut gs.vehicles[vehicle_handle];

            // Borrowchk dance - reborrow each iteration of the loop
            // so the borrow ends before we pass `gs` to other functions.
            let projectile = &gs.projectiles[proj_handle];

            if vehicle.destroyed() || vehicle.owner == projectile.owner {
                continue;
            }

            let nearest_point = step.projected_point(vehicle.pos);
            let dist2 = nearest_point.distance_squared(vehicle.pos);
            // TODO proper hitbox
            if dist2 <= 24.0 * 24.0 {
                if cvars.d_tracing {
                    dbg_cross!(nearest_point, 0.5);
                }
                let dmg = cvars.g_weapon_damage(projectile.weapon);
                let is_rail = projectile.weapon == Weapon::Rail;
                if is_rail {
                    gs.railguns.push((step.start, nearest_point));
                    vehicle.vel += step_dir * cvars.g_railgun_push;
                }

                // Vehicle explosion first so it's below projectile explosion because it looks better.
                let attacker_handle = projectile.owner;
                damage(cvars, gs, attacker_handle, vehicle_handle, dmg);
                if !is_rail {
                    projectile_impact(cvars, gs, proj_handle, nearest_point);
                    break; // TODO actually ... what if the segment is long and 2 vehicles are in the path
                }
            } else if projectile.weapon == Weapon::Rail {
                gs.railguns.push((step.start, step.end));
            } else if projectile.weapon == Weapon::Bfg
                && dist2 <= cvars.g_bfg_beam_range * cvars.g_bfg_beam_range
                && map.is_wall_trace(projectile.pos, vehicle.pos).is_none()
            {
                let dmg = cvars.g_bfg_beam_damage_per_sec * gs.dt;
                gs.bfg_beams.push((projectile.pos, vehicle.pos));
                let attacker_handle = projectile.owner;
                damage(cvars, gs, attacker_handle, vehicle_handle, dmg);
            }
        }

        if let Some(hit_pos) = maybe_collision {
            // Only hit the final wall if it didn't hit a vehicle first.
            // Otherwise this tries to remove the projectile a second time.
            //
            // We could set a flag when hitting vehicles above instead of `.contains` but this is more future-proof.
            if gs.projectiles.contains(proj_handle) {
                projectile_impact(cvars, gs, proj_handle, hit_pos);
            }
        }
    }
}

pub(crate) fn damage(
    cvars: &Cvars,
    gs: &mut GameState,
    attacker_handle: Index,
    vehicle_handle: Index,
    dmg_amount: f64,
) {
    let vehicle = &mut gs.vehicles[vehicle_handle];
    vehicle.hp_fraction -= dmg_amount / cvars.g_vehicle_hp(vehicle.veh_type);

    // Not using 0.0 here because of floating point errors.
    // Some weapons should reduce health to exact 0 in a small number of hits but it ends up being a tiny bit above it.
    if vehicle.hp_fraction > 0.001 {
        return;
    }

    // Vehicle got killed

    vehicle.hp_fraction = 0.0;
    gs.explosions
        .push(Explosion::new(vehicle.pos, 1.0, gs.frame_time, false));
    gs.players[vehicle.owner].guided_missile = None; // No guiding after death

    let attacker = &mut gs.players[attacker_handle];
    if attacker_handle == vehicle.owner {
        attacker.score.suicides += 1;
    } else {
        attacker.score.kills += 1;
    }
    let victim = &mut gs.players[vehicle.owner];
    victim.score.deaths += 1;
}

/// Right now, CBs are the only timed projectiles, long term, might wanna add timeouts to more
/// to avoid too many entities on huge maps.
pub(crate) fn projectiles_timeout(cvars: &Cvars, gs: &mut GameState) {
    for handle in gs.projectiles.iter_handles() {
        let projectile = &gs.projectiles[handle];
        if gs.frame_time > projectile.explode_time {
            let hit_pos = projectile.pos; // borrowchk dance
            projectile_impact(cvars, gs, handle, hit_pos);
        }
    }
}

fn projectile_impact(cvars: &Cvars, gs: &mut GameState, projectile_handle: Index, hit_pos: Vec2f) {
    let projectile = &mut gs.projectiles[projectile_handle];
    if let Some(expl_scale) = cvars.g_weapon_explosion_scale(projectile.weapon) {
        gs.explosions.push(Explosion::new(
            hit_pos,
            expl_scale,
            gs.frame_time,
            projectile.weapon == Weapon::Bfg,
        ));
    }
    if projectile.weapon == Weapon::Gm {
        let player = &mut gs.players[projectile.owner];
        if player.guided_missile == Some(projectile_handle) {
            player.guided_missile = None;
        }
    }
    gs.projectiles.remove(projectile_handle).unwrap();
}

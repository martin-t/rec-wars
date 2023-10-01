//! The S in ECS
//!
//! We're using the ECS design pattern (decouple behavior from data),
//! just without the ECS data structure (we use generational arenas instead).
//! Most game behavior (code that changes state) goes here.

use vek::LineSegment2;

use crate::prelude::*;

impl ServerFrameCtx<'_> {
    /// Delete data from previous frames that's no longer needed.
    pub fn sys_cleanup(&mut self) {
        // LATER Remove this if it remain unused
    }

    pub fn sys_respawning(&mut self) {
        for player_handle in self.gs.players.collect_handles() {
            let player = &mut self.gs.players[player_handle];
            let vehicle_handle = player.vehicle.unwrap();
            if !self.gs.vehicles[vehicle_handle].destroyed() {
                continue;
            }

            // Respawn on release so the vehicle doesn't immediately shoot.
            // Require the whole press and release cycle to happen while dead
            // so releasing fire after dying doesn't respawn immediately
            // even if respawn delay is 0.
            // LATER Allow press and release in one frame.

            if !player.input_prev.fire && player.input.fire {
                player.respawn = Respawn::Pressed;
            }

            if player.respawn == Respawn::Pressed && player.input_prev.fire && !player.input.fire {
                player.respawn = Respawn::Scheduled;
            }

            if player.respawn == Respawn::Scheduled
                && player.death_time + self.cvars.g_respawn_delay < self.gs.game_time
            {
                player.respawn = Respawn::No;
                self.gs.vehicles.remove(vehicle_handle).unwrap();
                self.spawn_vehicle(player_handle, true);
            }
        }
    }

    pub fn spawn_vehicle(&mut self, player_handle: Index, use_spawns: bool) {
        let veh_type = VehicleType::from_repr(self.sg.rng.gen_range(0..3)).unwrap();
        let (spawn_pos, spawn_angle) = if use_spawns {
            self.map.random_spawn(&mut self.sg.rng)
        } else {
            let (pos, _angle) = self.map.random_nonwall(&mut self.sg.rng);
            // Most grass tiles have no rotation so everyone ends up facing right which looks bad.
            // Throw away their angle and use a random one.
            let angle = self.sg.rng.gen_range(0.0..2.0 * PI);
            (pos, angle)
        };

        let vehicle_handle = self.gs.vehicles.insert(Vehicle::new(
            self.cvars,
            spawn_pos,
            spawn_angle,
            veh_type,
            self.gs.game_time,
            player_handle,
        ));

        let player = &mut self.gs.players[player_handle];
        player.vehicle = Some(vehicle_handle);

        let vehicle = &self.gs.vehicles[vehicle_handle];
        let init = VehicleInit {
            index: vehicle_handle.slot(),
            physics: EntityPhysics {
                pos: vehicle.pos,
                vel: vehicle.vel,
                angle: vehicle.angle,
                turn_rate: vehicle.turn_rate,
            },
            veh_type,
            turret_angle_current: vehicle.turret_angle_current,
            turret_angle_wanted: vehicle.turret_angle_wanted,
            spawn_time: vehicle.spawn_time,
            owner: vehicle.owner.slot(),
        };
        let msg = ServerMessage::SpawnVehicle(init);
        self.net_send_all(msg);
    }

    pub fn self_destruct(&mut self) {
        for vehicle_handle in self.gs.vehicles.collect_handles() {
            let vehicle = &self.gs.vehicles[vehicle_handle];
            let pos = vehicle.pos;
            let owner = vehicle.owner;
            let input = &self.gs.players[owner].input;
            if !input.self_destruct || vehicle.destroyed() {
                continue;
            }

            // 1) the big explosion
            self.spawn_explosion(pos, self.cvars.g_self_destruct_explosion_scale, false);

            // 2) all vehicles in range
            self.explosion_damage(
                owner,
                pos,
                self.cvars.g_self_destruct_damage_center,
                self.cvars.g_self_destruct_damage_edge,
                self.cvars.g_self_destruct_radius,
                Some(vehicle_handle),
            );

            // 3) the player vehicle to create the small explosion on top.
            self.damage(owner, vehicle_handle, f64::MAX);

            // LATER What was the order of explosions in the original RecWar? Make it configurable?
        }
    }

    pub fn sys_vehicle_movement(&mut self) {
        for (_, vehicle) in self.gs.vehicles.iter_mut() {
            let stats = self.cvars.g_vehicle_movement_stats(vehicle.veh_type);

            // No movement after death or when guiding
            let input = if vehicle.destroyed() {
                NetInput::empty()
            } else {
                let player = &self.gs.players[vehicle.owner];
                if player.guided_missile.is_some() {
                    self.gs.players[vehicle.owner].input.vehicle_while_guiding()
                } else {
                    self.gs.players[vehicle.owner].input
                }
            };
            let new_angle = Self::turning(
                &stats,
                &mut vehicle.vel,
                &vehicle.angle,
                &mut vehicle.turn_rate,
                input,
                self.gs.dt,
            );

            if vehicle
                .hitbox
                .corners(vehicle.pos, new_angle)
                .iter()
                .any(|&corner| self.map.is_wall(corner))
            {
                vehicle.turn_rate *= -0.5;
            } else {
                vehicle.angle = new_angle;
            }

            Self::accel_decel(
                &stats,
                &mut vehicle.vel,
                &mut vehicle.angle,
                input,
                self.gs.dt,
            );

            let new_pos = vehicle.pos + vehicle.vel * self.gs.dt;
            if vehicle
                .hitbox
                .corners(new_pos, vehicle.angle)
                .iter()
                .any(|&corner| self.map.is_wall(corner))
            {
                // LATER map edge in original RW absorbs the impact - there's no bounce
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
        input: NetInput,
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

    fn accel_decel(
        stats: &MovementStats,
        vel: &mut Vec2f,
        angle: &mut f64,
        input: NetInput,
        dt: f64,
    ) {
        let vel_change =
            (input.up() * stats.accel_forward - input.down() * stats.accel_backward) * dt;
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

    pub fn sys_player_weapon(&mut self) {
        for (_, player) in self.gs.players.iter_mut() {
            // Change weapon
            if !player.input_prev.prev_weapon && player.input.prev_weapon {
                let prev = (player.cur_weapon as usize + Weapon::COUNT - 1) % Weapon::COUNT;
                player.cur_weapon = Weapon::from_repr(prev).unwrap();
            }
            if !player.input_prev.next_weapon && player.input.next_weapon {
                let next = (player.cur_weapon as usize + 1) % Weapon::COUNT;
                player.cur_weapon = Weapon::from_repr(next).unwrap();
            }
        }
    }

    pub fn sys_vehicle_logic(&mut self) {
        for (_, vehicle) in self.gs.vehicles.iter_mut() {
            // This should run even while dead, otherwise the ammo indicator will be buggy.
            // Original RW also reloaded while dead.

            let player = &self.gs.players[vehicle.owner];

            // Turret turning
            if !player.input_prev.turret_left && player.input.turret_left {
                vehicle.turret_angle_wanted -= self.cvars.g_turret_turn_step_angle_deg.to_radians();
            }
            if !player.input_prev.turret_right && player.input.turret_right {
                vehicle.turret_angle_wanted += self.cvars.g_turret_turn_step_angle_deg.to_radians();
            }
            vehicle.turret_angle_wanted = vehicle.turret_angle_wanted.rem_euclid(2.0 * PI);

            let delta = vehicle
                .turret_angle_current
                .delta_angle(vehicle.turret_angle_wanted);
            let change =
                self.cvars.g_turret_turn_speed_deg.to_radians() * self.gs.dt * delta.signum();
            let change_clamped = change.clamped(-delta.abs(), delta.abs());
            vehicle.turret_angle_current += change_clamped;
            vehicle.turret_angle_current = vehicle.turret_angle_current.rem_euclid(2.0 * PI);

            // Reloading
            let ammo = &mut vehicle.ammos[player.cur_weapon as usize];
            if let Ammo::Reloading(_, end) = ammo {
                if self.gs.game_time >= *end {
                    *ammo = Ammo::Loaded(
                        self.gs.game_time,
                        self.cvars.g_weapon_reload_ammo(player.cur_weapon),
                    );
                }
            }
        }
    }

    pub fn sys_shooting(&mut self) {
        let mut new_projectiles = Vec::new();
        for (_, vehicle) in self.gs.vehicles.iter_mut() {
            let player = &mut self.gs.players[vehicle.owner];
            // Note: vehicles can shoot while controlling a missile
            if vehicle.destroyed() || !player.input.fire {
                continue;
            }

            let ammo = &mut vehicle.ammos[player.cur_weapon as usize];
            if let Ammo::Loaded(ready_time, count) = ammo {
                if self.gs.game_time < *ready_time {
                    continue;
                }

                *ready_time = self.gs.game_time + self.cvars.g_weapon_refire(player.cur_weapon);
                *count -= 1;
                if *count == 0 {
                    let reload_time = self.cvars.g_weapon_reload_time(player.cur_weapon);
                    *ammo = Ammo::Reloading(self.gs.game_time, self.gs.game_time + reload_time);
                }

                let (hardpoint, weapon_offset) =
                    self.cvars.g_hardpoint(vehicle.veh_type, player.cur_weapon);
                let (shot_angle, shot_origin);
                match hardpoint {
                    Hardpoint::Chassis => {
                        shot_angle = vehicle.angle;
                        shot_origin = vehicle.pos + weapon_offset.rotated_z(shot_angle);
                    }
                    Hardpoint::Turret => {
                        shot_angle = vehicle.angle + vehicle.turret_angle_current;
                        let turret_offset =
                            self.cvars.g_vehicle_turret_offset_chassis(vehicle.veh_type);
                        shot_origin = vehicle.pos
                            + turret_offset.rotated_z(vehicle.angle)
                            + weapon_offset.rotated_z(shot_angle);
                    }
                }

                // Some sane defaults to be overriden later
                let mut projectile = Projectile {
                    weapon: Weapon::Mg,
                    pos: shot_origin,
                    vel: Vec2f::zero(), // LATER hardpoint vel? -> more vel if vehicle is turning?
                    angle: shot_angle,
                    turn_rate: 0.0, // LATER vehicle turn angle?
                    explode_time: f64::MAX,
                    owner: vehicle.owner,
                    target: None,
                };

                match player.cur_weapon {
                    Weapon::Mg => {
                        let r: f64 = self.sg.rng.sample(StandardNormal);
                        let spread = self.cvars.g_machine_gun_angle_spread * r;
                        // Using spread as shot_vel.y would mean the resulting spread depends on speed
                        // so it's better to use spread on angle.
                        projectile.vel = Vec2f::new(self.cvars.g_machine_gun_speed, 0.0)
                            .rotated_z(shot_angle + spread)
                            + self.cvars.g_machine_gun_vehicle_velocity_factor * vehicle.vel;
                        let handle = self.gs.projectiles.insert(projectile);
                        new_projectiles.push(handle);
                    }
                    Weapon::Rail => {
                        projectile.weapon = Weapon::Rail;
                        projectile.vel = Vec2f::new(self.cvars.g_railgun_speed, 0.0)
                            .rotated_z(shot_angle)
                            + self.cvars.g_railgun_vehicle_velocity_factor * vehicle.vel;
                        let handle = self.gs.projectiles.insert(projectile);
                        new_projectiles.push(handle);
                    }
                    Weapon::Cb => {
                        projectile.weapon = Weapon::Cb;
                        for _ in 0..self.cvars.g_cluster_bomb_count {
                            let speed = self.cvars.g_cluster_bomb_speed;
                            let spread_forward;
                            let spread_sideways;
                            if self.cvars.g_cluster_bomb_speed_spread_gaussian {
                                let r: f64 = self.sg.rng.sample(StandardNormal);
                                spread_forward = self.cvars.g_cluster_bomb_speed_spread_forward * r;
                                let r: f64 = self.sg.rng.sample(StandardNormal);
                                spread_sideways =
                                    self.cvars.g_cluster_bomb_speed_spread_sideways * r;
                            } else {
                                let r = self.sg.rng.sample(self.gs.range_uniform11);
                                spread_forward = self.cvars.g_cluster_bomb_speed_spread_forward * r;
                                let r = self.sg.rng.sample(self.gs.range_uniform11);
                                spread_sideways =
                                    self.cvars.g_cluster_bomb_speed_spread_sideways * r;
                            }
                            projectile.vel = Vec2f::new(speed + spread_forward, spread_sideways)
                                .rotated_z(shot_angle)
                                + self.cvars.g_cluster_bomb_vehicle_velocity_factor * vehicle.vel;
                            projectile.explode_time = self.gs.game_time
                                + self.cvars.g_cluster_bomb_time
                                + self.sg.rng.sample(self.gs.range_uniform11)
                                    * self.cvars.g_cluster_bomb_time_spread;
                            let handle = self.gs.projectiles.insert(projectile.clone());
                            new_projectiles.push(handle);
                        }
                    }
                    Weapon::Rockets => {
                        projectile.weapon = Weapon::Rockets;
                        projectile.vel = Vec2f::new(self.cvars.g_rockets_speed, 0.0)
                            .rotated_z(shot_angle)
                            + self.cvars.g_rockets_vehicle_velocity_factor * vehicle.vel;
                        let handle = self.gs.projectiles.insert(projectile);
                        new_projectiles.push(handle);
                    }
                    Weapon::Hm => {
                        projectile.weapon = Weapon::Hm;
                        projectile.vel = Vec2f::new(self.cvars.g_homing_missile_speed_initial, 0.0)
                            .rotated_z(shot_angle)
                            + self.cvars.g_homing_missile_vehicle_velocity_factor * vehicle.vel;
                        let handle = self.gs.projectiles.insert(projectile);
                        new_projectiles.push(handle);
                    }
                    Weapon::Gm => {
                        projectile.weapon = Weapon::Gm;
                        projectile.vel = Vec2f::new(self.cvars.g_guided_missile_speed_initial, 0.0)
                            .rotated_z(shot_angle)
                            + self.cvars.g_guided_missile_vehicle_velocity_factor * vehicle.vel;
                        // LATER Set angle according to vehicle angle (also some other weaps)
                        let handle = self.gs.projectiles.insert(projectile);
                        new_projectiles.push(handle);
                        player.guided_missile = Some(handle);
                    }
                    Weapon::Bfg => {
                        projectile.weapon = Weapon::Bfg;
                        projectile.vel = Vec2f::new(self.cvars.g_bfg_speed, 0.0)
                            .rotated_z(shot_angle)
                            + self.cvars.g_bfg_vehicle_velocity_factor * vehicle.vel;
                        let handle = self.gs.projectiles.insert(projectile);
                        new_projectiles.push(handle);
                    }
                }
            }
        }

        for handle in new_projectiles {
            let projectile = &self.gs.projectiles[handle];
            let spawn = ProjectileInit {
                index: handle.slot(),
                weapon: projectile.weapon,
                physics: EntityPhysics {
                    pos: projectile.pos,
                    vel: projectile.vel,
                    angle: projectile.angle,
                    turn_rate: projectile.turn_rate,
                },
                explode_time: projectile.explode_time,
                owner: projectile.owner.slot(),
            };
            let msg = ServerMessage::SpawnProjectile(spawn);
            self.net_send_all(msg);
        }
    }

    fn hm_forget(hm_handle: Index, hm: &mut Projectile, target: &mut Vehicle) {
        hm.target = None;
        let index = target.hms.iter().position(|&h| h == hm_handle).unwrap();
        target.hms.swap_remove(index);
    }

    /// The *homing* part of homing missile
    pub fn sys_hm_turning(&mut self) {
        for (hm_handle, hm) in self
            .gs
            .projectiles
            .iter_mut()
            .filter(|(_, proj)| proj.weapon == Weapon::Hm)
        {
            // Forget target under some conditions
            if let Some(target_handle) = hm.target {
                if !self.gs.vehicles.contains(target_handle) {
                    // Vehicle is gone (player disconnected)
                    hm.target = None;
                    continue;
                }

                let target = &mut self.gs.vehicles[target_handle];
                if target.destroyed() {
                    Self::hm_forget(hm_handle, hm, target);
                    continue;
                }

                let target_dir = (target.pos - hm.pos).normalized();
                let target_angle = target_dir.to_angle();

                let mut angle_diff = (target_angle - hm.angle).rem_euclid(2.0 * PI);
                if angle_diff > PI {
                    angle_diff -= 2.0 * PI;
                }
                if angle_diff.abs() > self.cvars.g_homing_missile_angle_forget {
                    Self::hm_forget(hm_handle, hm, target);
                }
            }

            // Pick new target
            if hm.target.is_none() {
                // Where the missile is aiming.
                // Not using velocity because it can move sieways sometimes (especially during launch).
                let forward_dir = hm.angle.to_vec2f();

                let mut best_target = None;
                let mut best_target_angle_diff = f64::INFINITY;

                for (vehicle_handle, vehicle) in self.gs.vehicles.iter() {
                    if vehicle.owner == hm.owner || vehicle.destroyed() {
                        // LATER Allow targetting self if the missile loops around other vehicles
                        continue;
                    }

                    let target_dir = (vehicle.pos - hm.pos).normalized();
                    let dot = forward_dir.dot(target_dir);
                    let angle_diff = dot.acos();
                    if angle_diff < self.cvars.g_homing_missile_angle_detect
                        && angle_diff < best_target_angle_diff
                        && self.map.is_wall_trace(hm.pos, vehicle.pos).is_none()
                    {
                        best_target = Some(vehicle_handle);
                        best_target_angle_diff = angle_diff;
                    }
                }

                if let Some(best_target) = best_target {
                    hm.target = Some(best_target);
                    self.gs.vehicles[best_target].hms.push(hm_handle);
                }
            }

            // Determine direction
            let mut input = NetInput::new_up();
            if let Some(target_handle) = hm.target {
                let target = &self.gs.vehicles[target_handle];
                let target_dir = (target.pos - hm.pos).normalized();
                let target_angle = target_dir.to_angle();

                let angle_diff = (target_angle - hm.angle).rem_euclid(2.0 * PI);
                if angle_diff < PI {
                    input.right = true;
                } else {
                    input.left = true;
                }

                // LATER Use https://crates.io/crates/pid ?
            }

            // Movement
            let stats = self.cvars.g_homing_missile_movement_stats();
            hm.angle = Self::turning(
                &stats,
                &mut hm.vel,
                &hm.angle,
                &mut hm.turn_rate,
                input,
                self.gs.dt,
            );
            Self::accel_decel(&stats, &mut hm.vel, &mut hm.angle, input, self.gs.dt);
        }
    }

    /// The *guided* part of guided missile
    pub fn sys_gm_turning(&mut self) {
        for (gm_handle, gm) in self
            .gs
            .projectiles
            .iter_mut()
            .filter(|(_, proj)| proj.weapon == Weapon::Gm)
        {
            let stats = self.cvars.g_guided_missile_movement_stats();
            let player = &self.gs.players[gm.owner];

            // Only allow guiding the most recently launched missile
            let input = if player.guided_missile == Some(gm_handle) {
                player.input.missile_while_guiding()
            } else {
                NetInput::new_up()
            };

            gm.angle = Self::turning(
                &stats,
                &mut gm.vel,
                &gm.angle,
                &mut gm.turn_rate,
                input,
                self.gs.dt,
            );

            Self::accel_decel(&stats, &mut gm.vel, &mut gm.angle, input, self.gs.dt);
        }
    }

    /// Projectile movement and collisions / hit detection.
    /// Traces the projectile's path between positions to avoid passing through thin objects.
    pub fn sys_projectiles(&mut self) {
        for proj_handle in self.gs.projectiles.collect_handles() {
            let projectile = &mut self.gs.projectiles[proj_handle];
            let max_new_pos = projectile.pos + projectile.vel * self.gs.dt;

            if projectile.weapon == Weapon::Cb {
                projectile.pos = max_new_pos;
                continue;
            }

            let maybe_collision = self.map.is_wall_trace(projectile.pos, max_new_pos);
            let new_pos = if let Some(hit_pos) = maybe_collision {
                hit_pos
            } else {
                max_new_pos
            };

            if self.cvars.d_tracing {
                dbg_line!(projectile.pos, new_pos, 0.5);
            }
            let step = LineSegment2 {
                start: projectile.pos,
                end: new_pos,
            };
            let step_dir = (new_pos - projectile.pos).normalized();

            projectile.pos = new_pos;
            if self.cvars.d_projectiles {
                dbg_cross!(projectile.pos);
            }

            let is_rail = projectile.weapon == Weapon::Rail;
            if is_rail {
                let beam = RailBeam::new(step.start, step.end, self.gs.game_time);
                let msg = ServerMessage::RailBeam(beam);
                self.net_send_all(msg);
            }

            for vehicle_handle in self.gs.vehicles.collect_handles() {
                // LATER immediately killing vehicles here means 2 players can't share a kill
                let vehicle = &mut self.gs.vehicles[vehicle_handle];

                // borrowck dance - reborrow each iteration of the loop
                // so the borrow ends before we pass `gs` to other functions.
                let projectile = &self.gs.projectiles[proj_handle];

                if vehicle.destroyed()
                    || vehicle.owner == projectile.owner
                    || (is_rail && self.gs.rail_hits.get(&proj_handle) == Some(&vehicle_handle))
                {
                    continue;
                }

                let nearest_point = step.projected_point(vehicle.pos);
                let dist2 = nearest_point.distance_squared(vehicle.pos);
                if dist2 <= self.cvars.g_hitcircle_radius * self.cvars.g_hitcircle_radius {
                    if self.cvars.d_tracing {
                        dbg_cross!(nearest_point, 0.5);
                    }
                    let dmg = self.cvars.g_weapon_damage_direct(projectile.weapon);

                    if is_rail {
                        self.gs.rail_hits.insert(proj_handle, vehicle_handle);
                        vehicle.vel += step_dir * self.cvars.g_railgun_push;
                    }

                    let attacker_handle = projectile.owner;
                    self.damage(attacker_handle, vehicle_handle, dmg);
                    if !is_rail {
                        self.projectile_impact(proj_handle, nearest_point);
                        break; // LATER actually ... what if the segment is long and 2 vehicles are in the path
                    }
                } else if projectile.weapon == Weapon::Bfg
                    && weapons::bfg_beam_hit(&self.cvars, &self.map, projectile.pos, vehicle.pos)
                {
                    let dmg = self.cvars.g_bfg_beam_damage_per_sec * self.gs.dt;
                    let attacker_handle = projectile.owner;
                    self.damage(attacker_handle, vehicle_handle, dmg);
                }
            }

            if let Some(hit_pos) = maybe_collision {
                // Only hit the final wall if it didn't hit a vehicle first.
                // Otherwise this tries to remove the projectile a second time.
                // We could set a flag when hitting vehicles above instead of `.contains` but this is more future-proof.
                if self.gs.projectiles.contains(proj_handle) {
                    self.projectile_impact(proj_handle, hit_pos);
                    if is_rail {
                        self.gs.rail_hits.remove(&proj_handle);
                    }
                }
            }
        }
    }

    pub fn damage(&mut self, attacker_handle: Index, vehicle_handle: Index, dmg_amount: f64) {
        let vehicle = &mut self.gs.vehicles[vehicle_handle];

        soft_assert!(!vehicle.destroyed());

        vehicle.hp_fraction -= dmg_amount / self.cvars.g_vehicle_hp(vehicle.veh_type);

        // Not using 0.0 here because of floating point errors.
        // Some weapons should reduce health to exact 0 in a small number of hits but it ends up being a tiny bit above it.
        if vehicle.hp_fraction > 0.001 {
            return;
        }

        // Vehicle got killed

        vehicle.hp_fraction = 0.0;
        let veh_owner = vehicle.owner; // Borrowck
        let veh_pos = vehicle.pos; // Borrowck
        self.spawn_explosion(veh_pos, 1.0, false);
        self.gs.players[veh_owner].guided_missile = None; // No guiding after death

        if self.cvars.d_log_kills {
            // Indent kill msgs because there's a lot of them so others stand out.
            // LATER configurable indent
            let attacker_name = &self.gs.players[attacker_handle].name.clone();
            let victim_name = &self.gs.players[veh_owner].name.clone();
            dbg_logf!("    {victim_name:?} was killed by {attacker_name:?}");
        }

        let victim = &mut self.gs.players[veh_owner];
        victim.death_time = self.gs.game_time;

        self.update_score_kill(attacker_handle, veh_owner);

        let kill = Kill {
            attacker: attacker_handle.slot(),
            victim: veh_owner.slot(),
        };
        let msg = ServerMessage::Kill(kill);
        self.net_send_all(msg);
    }

    /// Right now, CBs are the only timed projectiles, long term, might wanna add timeouts to more
    /// to avoid too many entities on huge maps.
    pub fn sys_projectiles_timeout(&mut self) {
        for handle in self.gs.projectiles.collect_handles() {
            let projectile = &self.gs.projectiles[handle];
            if self.gs.game_time > projectile.explode_time {
                let hit_pos = projectile.pos; // borrowck dance
                self.projectile_impact(handle, hit_pos);
            }
        }
    }

    pub fn spawn_explosion(&mut self, pos: Vec2f, scale: f64, bfg: bool) {
        if scale == 0.0 {
            return;
        }

        let init = ExplosionInit { pos, scale, bfg };
        let msg = ServerMessage::SpawnExplosion(init);
        self.net_send_all(msg);
    }

    // LATER This shouldn't need to take hit_pos
    fn projectile_impact(&mut self, projectile_handle: Index, hit_pos: Vec2f) {
        let projectile = &mut self.gs.projectiles[projectile_handle];

        // borrowck dance
        let weapon = projectile.weapon;
        let owner = projectile.owner;
        let target = projectile.target;

        // Vehicle explosion first so it's below projectile explosion because it looks better.
        let expl_scale = self.cvars.g_weapon_explosion_scale(weapon);
        let expl_bfg = weapon == Weapon::Bfg;
        self.spawn_explosion(hit_pos, expl_scale, expl_bfg);

        let expl_damage = expl_scale * self.cvars.g_weapon_explosion_damage(weapon);
        let expl_radius = expl_scale * self.cvars.g_weapon_explosion_radius(weapon);
        if expl_damage > 0.0 || expl_radius > 0.0 {
            self.explosion_damage(owner, hit_pos, expl_damage, expl_damage, expl_radius, None);
        }

        if weapon == Weapon::Hm {
            if let Some(target) = target {
                let target = &mut self.gs.vehicles[target];
                // Borrowck dance:
                // No need to hm_forget here because the projectile is desotryed anyway.
                // We actually can't call it anyway because we can't keep projectile borrowed.
                let index = target
                    .hms
                    .iter()
                    .position(|&h| h == projectile_handle)
                    .unwrap();
                target.hms.swap_remove(index);
            }
        }

        if weapon == Weapon::Gm {
            let player = &mut self.gs.players[owner];
            if player.guided_missile == Some(projectile_handle) {
                player.guided_missile = None;
            }
        }

        let msg = ServerMessage::DestroyProjectile {
            index: projectile_handle.slot(),
        };
        self.net_send_all(msg);
        self.gs.projectiles.remove(projectile_handle).unwrap();
    }

    fn explosion_damage(
        &mut self,
        owner: Index,
        expl_pos: Vec2f,
        damage_center: f64,
        damage_edge: f64,
        radius: f64,
        ignore: Option<Index>,
    ) {
        if self.cvars.d_explosion_radius {
            dbg_line!(expl_pos, expl_pos + Vec2f::new(radius, 0.0), 5.0);
        }

        for vehicle_handle in self.gs.vehicles.collect_handles() {
            if let Some(ignore) = ignore {
                if vehicle_handle == ignore {
                    continue;
                }
            }

            let vehicle = &self.gs.vehicles[vehicle_handle];
            if vehicle.destroyed() {
                continue;
            }

            let center_dist = (vehicle.pos - expl_pos).magnitude();
            let dist = (center_dist - self.cvars.g_hitcircle_radius).max(0.0);
            if dist < radius {
                let expl_damage = lerp_ranges(0.0, radius, damage_center, damage_edge, dist);
                self.damage(owner, vehicle_handle, expl_damage);
            }
        }
    }
}

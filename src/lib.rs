// Additional warnings that are allow by default (`rustc -W help`)
//#![warn(missing_copy_implementations)]
//#![warn(missing_debug_implementations)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
//#![warn(unreachable_pub)]
#![warn(unused)]
#![warn(clippy::all)]

#[macro_use]
mod debugging;

mod components;
mod cvars;
mod entities;
mod game_state;
mod map;
mod weapons;

use std::collections::VecDeque;
use std::f64::consts::PI;

use hecs;

use legion;
use legion::query::IntoQuery;

use js_sys::Array;

use rand::prelude::*;
use rand_distr::StandardNormal;

use vek::ops::Clamp;
use vek::Vec2;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::{CanvasRenderingContext2d, HtmlImageElement, Performance};

use components::{Angle, Cb, Hitbox, Pos, Rocket, Time, Vel};
use cvars::{Cvars, TickrateMode};
use debugging::{DEBUG_CROSSES, DEBUG_LINES, DEBUG_TEXTS};
use entities::{Ammo, GuidedMissile, Tank};
use game_state::{Explosion, GameState, Input, PlayerEntity};
use map::{F64Ext, Kind, Map, Vec2f, VecExt, TILE_SIZE};
use weapons::*;

#[wasm_bindgen]
pub struct Game {
    /// I want to track update and render time in Rust so i can draw the FPS counter and keep stats.
    /// Unfortunately, Instant::now() panics in WASM so i have to use performance.now().
    /// And just like in JS, it has limited precision in some browsers like firefox.
    performance: Performance,
    context: CanvasRenderingContext2d,
    canvas_size: Vec2f,
    imgs_textures: Vec<HtmlImageElement>,
    imgs_vehicles: Vec<HtmlImageElement>,
    imgs_weapon_icons: Vec<HtmlImageElement>,
    img_rocket: HtmlImageElement,
    img_gm: HtmlImageElement,
    img_tank_green: HtmlImageElement,
    img_tank_red: HtmlImageElement,
    img_explosion: HtmlImageElement,
    /// Saved frame times in seconds over some period of time to measure FPS
    frame_times: VecDeque<f64>,
    update_durations: VecDeque<f64>,
    draw_durations: VecDeque<f64>,
    map: Map,
    gs: GameState,
    gs_prev: GameState,
    hecs: hecs::World,
    legion: legion::World,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(
        cvars: &Cvars,
        context: CanvasRenderingContext2d,
        width: f64,
        height: f64,
        textures: Array,
        vehicles: Array,
        weapon_icons: Array,
        img_rocket: HtmlImageElement,
        img_gm: HtmlImageElement,
        img_tank_green: HtmlImageElement,
        img_tank_red: HtmlImageElement,
        img_explosion: HtmlImageElement,
        tex_list_text: &str,
        map_text: &str,
    ) -> Self {
        console_error_panic_hook::set_once();

        let mut rng = if cvars.d_seed == 0 {
            // This requires the `wasm-bindgen` feature on `rand` or it crashes at runtime.
            SmallRng::from_entropy()
        } else {
            SmallRng::seed_from_u64(cvars.d_seed)
        };

        let imgs_textures = textures
            .iter()
            .map(|tile| tile.dyn_into().unwrap())
            .collect();
        let imgs_vehicles = vehicles
            .iter()
            .map(|js_val| js_val.dyn_into().unwrap())
            .collect();
        let imgs_weapon_icons = weapon_icons
            .iter()
            .map(|js_val| js_val.dyn_into().unwrap())
            .collect();

        let surfaces = map::load_tex_list(tex_list_text);
        let map = map::load_map(map_text, surfaces);
        let (pos, angle) = map.random_spawn(&mut rng);

        let gm = GuidedMissile::spawn(cvars, pos, angle);
        let tank = Tank::spawn(cvars, pos, angle);
        let pe = PlayerEntity::Tank;

        let mut gs = GameState {
            rng,
            frame_time: 0.0,
            input: Input::default(),
            cur_weapon: 5,
            railguns: Vec::new(),
            gm,
            tank,
            pe,
            explosions: Vec::new(),
        };
        let gs_prev = gs.clone();

        let mut legion = legion::World::default();
        for _ in 0..50 {
            let pos = map.random_nonwall(&mut gs.rng).0;
            let mins = Vec2f::new(cvars.g_tank_mins_x, cvars.g_tank_mins_y);
            let maxs = Vec2f::new(cvars.g_tank_maxs_x, cvars.g_tank_maxs_y);
            let angle = gs.rng.gen_range(0.0, 2.0 * PI);
            legion.push((Pos(pos), Angle(angle), Hitbox::new(mins, maxs)));
        }

        Self {
            performance: web_sys::window().unwrap().performance().unwrap(),
            context,
            canvas_size: Vec2f::new(width, height),
            imgs_textures,
            imgs_vehicles,
            imgs_weapon_icons,
            img_rocket,
            img_gm,
            img_tank_green,
            img_tank_red,
            img_explosion,
            frame_times: VecDeque::new(),
            update_durations: VecDeque::new(),
            draw_durations: VecDeque::new(),
            map,
            gs,
            gs_prev,
            hecs: hecs::World::new(),
            legion,
        }
    }

    /// Run gamelogic up to `t` (in seconds) and render.
    pub fn update_and_draw(&mut self, t: f64, input: &Input, cvars: &Cvars) -> Result<(), JsValue> {
        self.update(t, input, cvars);
        self.draw(cvars)?;
        Ok(())
    }

    pub fn update(&mut self, t: f64, input: &Input, cvars: &Cvars) {
        // Recommended reading: https://gafferongames.com/post/fix_your_timestep/

        let start = self.performance.now();

        // TODO prevent death spirals
        match cvars.sv_gamelogic_mode {
            TickrateMode::Synchronized => {
                self.begin_frame(t);
                self.input(input);
                self.tick(cvars);
            }
            TickrateMode::SynchronizedBounded => todo!(),
            TickrateMode::Fixed => loop {
                // gs, not gs_prev, is the previous frame here
                let remaining = t - self.gs.frame_time;
                let dt = 1.0 / cvars.sv_gamelogic_fixed_fps;
                if remaining < dt {
                    break;
                }
                self.begin_frame(self.gs.frame_time + dt);
                self.input(input);
                self.tick(cvars);
            },
            TickrateMode::FixedOrSmaller => todo!(),
            TickrateMode::FixedWithInterpolation => todo!(),
        }

        let end = self.performance.now();
        if self.update_durations.len() >= 60 {
            self.update_durations.pop_front();
        }
        self.update_durations.push_back(end - start);
    }

    /// Update time tracking variables (in seconds)
    fn begin_frame(&mut self, t: f64) {
        self.gs_prev = self.gs.clone();
        self.gs.frame_time = t;
        assert!(
            self.gs.frame_time >= self.gs_prev.frame_time,
            "frametime didn't increase: prev {}, current {}",
            self.gs_prev.frame_time,
            self.gs.frame_time,
        );

        self.frame_times.push_back(t);
        while !self.frame_times.is_empty() && self.frame_times.front().unwrap() + 1.0 < t {
            self.frame_times.pop_front();
        }
    }

    fn input(&mut self, input: &Input) {
        self.gs.input = input.clone();
    }

    fn tick(&mut self, cvars: &Cvars) {
        let frame_time = self.gs.frame_time; // borrowchk
        let dt = frame_time - self.gs_prev.frame_time;

        // Cleanup old entities
        self.gs.explosions.retain(|explosion| {
            let progress = (frame_time - explosion.start_time) / cvars.r_explosion_duration;
            progress <= 1.0
        });

        // Change weapon
        if self.gs.input.prev_weapon && !self.gs_prev.input.prev_weapon {
            self.gs.cur_weapon = (self.gs.cur_weapon + WEAPS_CNT - 1) % WEAPS_CNT;
        }
        if self.gs.input.next_weapon && !self.gs_prev.input.next_weapon {
            self.gs.cur_weapon = (self.gs.cur_weapon + 1) % WEAPS_CNT;
        }

        // Reloading
        let cur_weap = self.gs.cur_weapon;
        let ammo = &mut self.gs.tank.ammos[cur_weap];
        if let Ammo::Reloading(_, end) = ammo {
            if frame_time >= *end {
                *ammo = Ammo::Loaded(frame_time, cvars.g_weapon_reload_ammo(cur_weap));
            }
        }

        // Firing
        // Note: vehicles can shoot while controlling a missile
        if self.gs.input.fire {
            if let Ammo::Loaded(ready_time, count) = ammo {
                if frame_time >= *ready_time {
                    *ready_time = frame_time + cvars.g_weapon_refire(cur_weap);
                    *count -= 1;
                    if *count == 0 {
                        *ammo = Ammo::Reloading(frame_time, frame_time + 1.0);
                    }

                    match self.gs.cur_weapon {
                        WEAP_MG => {
                            // TODO move to MG
                            let pos = Pos(self.gs.tank.pos);
                            let mut vel = Vec2f::new(cvars.g_machine_gun_speed, 0.0)
                                .rotated_z(self.gs.tank.angle);
                            if cvars.g_machine_gun_add_vehicle_velocity {
                                vel += self.gs.tank.vel;
                            }
                            let vel = Vel(vel);
                            self.hecs.spawn((pos, vel));
                        }
                        WEAP_RAIL => {
                            let begin = self.gs.tank.pos;
                            let end = begin + self.gs.tank.angle.to_vec2f() * 100_000.0;
                            let hit = self.map.collision_between(begin, end);
                            if let Some(hit) = hit {
                                self.gs.railguns.push((begin, hit));
                            }
                        }
                        WEAP_CB => {
                            let pos = Pos(self.gs.tank.pos);
                            for _ in 0..cvars.g_cluster_bomb_count {
                                let speed = cvars.g_cluster_bomb_speed;
                                let spread_forward;
                                let spread_sideways;
                                if cvars.g_cluster_bomb_speed_spread_gaussian {
                                    // Broken type inference (works with rand crate but distributions are deprecated).
                                    let r: f64 = self.gs.rng.sample(StandardNormal);
                                    spread_forward = cvars.g_cluster_bomb_speed_spread_forward * r;
                                    let r: f64 = self.gs.rng.sample(StandardNormal);
                                    spread_sideways =
                                        cvars.g_cluster_bomb_speed_spread_sideways * r;
                                } else {
                                    let r = self.gs.rng.gen_range(-1.5, 1.5);
                                    spread_forward = cvars.g_cluster_bomb_speed_spread_forward * r;
                                    let r = self.gs.rng.gen_range(-1.5, 1.5);
                                    spread_sideways =
                                        cvars.g_cluster_bomb_speed_spread_sideways * r;
                                }

                                let mut vel = Vec2f::new(speed + spread_forward, spread_sideways)
                                    .rotated_z(self.gs.tank.angle);
                                if cvars.g_cluster_bomb_add_vehicle_velocity {
                                    vel += self.gs.tank.vel;
                                }
                                let vel = Vel(vel);
                                let time = frame_time
                                    + cvars.g_cluster_bomb_time
                                    + self.gs.rng.gen_range(-1.0, 1.0)
                                        * cvars.g_cluster_bomb_time_spread;
                                let time = Time(time);
                                self.legion.push((Cb, pos, vel, time));
                            }
                        }
                        WEAP_ROCKETS => {
                            // TODO move to turret end
                            let pos = Pos(self.gs.tank.pos);
                            let mut vel = Vec2f::new(cvars.g_rockets_speed, 0.0)
                                .rotated_z(self.gs.tank.angle);
                            if cvars.g_rockets_add_vehicle_velocity {
                                vel += self.gs.tank.vel;
                            }
                            let vel = Vel(vel);
                            self.legion.push((Rocket, pos, vel));
                        }
                        WEAP_HM => {}
                        WEAP_GM => {
                            self.gs.gm =
                                GuidedMissile::spawn(cvars, self.gs.tank.pos, self.gs.tank.angle);
                            self.gs.pe = PlayerEntity::GuidedMissile;
                        }
                        WEAP_BFG => {}
                        _ => unreachable!("current weapon index out of range"),
                    }
                }
            }
        }

        // MG
        let mut to_remove = Vec::new();
        for (entity, (pos, vel)) in self.hecs.query::<(&mut Pos, &Vel)>().iter() {
            pos.0 += vel.0 * dt;

            if self.map.collision(pos.0) {
                to_remove.push(entity);
            }
        }
        for entity in to_remove {
            self.hecs.despawn(entity).unwrap();
        }

        let mut to_remove = Vec::new();

        // CBs
        let mut query = <(legion::Entity, &Cb, &mut Pos, &Vel, &Time)>::query();
        for (&entity, _, pos, vel, time) in query.iter_mut(&mut self.legion) {
            pos.0 += vel.0 * dt;

            if frame_time > time.0 {
                self.gs.explosions.push(Explosion::new(
                    pos.0,
                    cvars.g_cluster_bomb_explosion_scale,
                    time.0,
                ));
                to_remove.push(entity);
            }
        }

        // Rockets
        let mut query_vehicles = <(&Pos, &Angle, &Hitbox)>::query();
        let vehicles: Vec<_> = query_vehicles
            .iter(&self.legion)
            .map(|(&pos, &angle, &hitbox)| (pos, angle, hitbox))
            .collect();

        let mut query = <(legion::Entity, &Rocket, &mut Pos, &Vel)>::query();
        for (&entity, _, pos, vel) in query.iter_mut(&mut self.legion) {
            pos.0 += vel.0 * dt;

            if self.map.collision(pos.0) {
                self.gs.explosions.push(Explosion::new(
                    pos.0,
                    cvars.g_rockets_explosion_scale,
                    self.gs.frame_time,
                ));
                to_remove.push(entity);
                continue;
            }

            for (veh_pos, _veh_angle, _veh_hitbox) in &vehicles {
                if (pos.0 - veh_pos.0).magnitude_squared() <= 24.0 * 24.0 {
                    self.gs.explosions.push(Explosion::new(
                        pos.0,
                        cvars.g_rockets_explosion_scale,
                        self.gs.frame_time,
                    ));
                    to_remove.push(entity);
                    break;
                }
            }
        }

        for entity in to_remove {
            self.legion.remove(entity);
        }

        // Movement
        if self.gs.pe == PlayerEntity::Tank {
            self.gs.tank.tick(dt, cvars, &self.gs.input, &self.map);
        } else {
            self.gs.tank.tick(dt, cvars, &Input::default(), &self.map);
        };

        let hit_something = if self.gs.pe == PlayerEntity::GuidedMissile {
            self.gs.gm.tick(dt, cvars, &self.gs.input, &self.map)
        } else {
            self.gs.gm.tick(dt, cvars, &Input::default(), &self.map)
        };
        if hit_something {
            let explosion = Explosion::new(self.gs.gm.pos, 1.0, self.gs.frame_time);
            self.gs.explosions.push(explosion);
            self.gs.pe = PlayerEntity::Tank;
            let (pos, angle) = self.map.random_spawn(&mut self.gs.rng);
            self.gs.gm = GuidedMissile::spawn(cvars, pos, angle);
        }
    }

    pub fn draw(&mut self, cvars: &Cvars) -> Result<(), JsValue> {
        let start = self.performance.now();

        // Nicer rockets (more like original RW).
        // This also means everything is aligned to pixels
        // without the need to explicitly round x and y in draw calls to whole numbers.
        // TODO revisit when drawing tanks - maybe make configurable per drawn object
        //      if disabling, try changing quality
        self.context.set_image_smoothing_enabled(cvars.r_smoothing);

        let pe_pos = match self.gs.pe {
            PlayerEntity::Tank => self.gs.tank.pos,
            PlayerEntity::GuidedMissile => self.gs.gm.pos,
        };

        // Don't put the camera so close to the edge that it would render area outside the map.
        // TODO handle maps smaller than canvas (currently crashes on unreachable)
        let camera_min = self.canvas_size / 2.0;
        let map_size = self.map.maxs();
        let camera_max = map_size - camera_min;
        let camera_pos = pe_pos.clamped(camera_min, camera_max);

        let top_left = camera_pos - camera_min;
        let top_left_tp = self.map.tile_pos(top_left);
        let top_left_index = top_left_tp.index;
        let bg_offset = if cvars.r_align_to_pixels_background {
            top_left_tp.offset.floor()
        } else {
            top_left_tp.offset
        };
        // TODO align player? other?

        // Draw non-walls
        let mut r = top_left_index.y;
        let mut y = -bg_offset.y;
        while y < self.canvas_size.y {
            let mut c = top_left_index.x;
            let mut x = -bg_offset.x;
            while x < self.canvas_size.x {
                let tile = self.map.col_row(c, r);

                if self.map.surface_of(tile).kind != Kind::Wall {
                    let img = &self.imgs_textures[tile.surface_index];
                    self.draw_img_top_left(img, Vec2::new(x, y), tile.angle)?;
                }

                c += 1;
                x += TILE_SIZE;
            }
            r += 1;
            y += TILE_SIZE;
        }

        // Draw MGs
        self.context.set_stroke_style(&"yellow".into());
        let mut mg_cnt = 0;
        for (_, (pos, vel)) in self.hecs.query::<(&Pos, &Vel)>().iter() {
            mg_cnt += 1;
            let scr_pos = pos.0 - top_left;
            self.context.begin_path();
            self.context.move_to(scr_pos.x, scr_pos.y);
            // we're drawing from the bullet's position backwards
            let scr_end = scr_pos - vel.0.normalized() * cvars.g_machine_gun_trail_length;
            self.line_to(scr_end);
            self.context.stroke();
        }
        dbg_textd!(mg_cnt);

        // Draw railguns
        self.context.set_stroke_style(&"blue".into());
        for (begin, end) in &self.gs.railguns {
            let scr_src = begin - top_left;
            let scr_hit = end - top_left;
            self.context.begin_path();
            self.move_to(scr_src);
            self.line_to(scr_hit);
            self.context.stroke();
        }
        self.gs.railguns.clear();

        // Draw cluster bombs
        if cvars.r_draw_cluster_bombs {
            self.context.set_fill_style(&"rgb(0, 255, 255)".into());
            let shadow_rgba = format!("rgba(0, 0, 0, {})", cvars.g_cluster_bomb_shadow_alpha);
            self.context.set_shadow_color(&shadow_rgba);
            self.context
                .set_shadow_offset_x(cvars.g_cluster_bomb_shadow_x);
            self.context
                .set_shadow_offset_y(cvars.g_cluster_bomb_shadow_y);
            let mut cb_cnt = 0;
            let mut query = <(&Cb, &Pos)>::query();
            for (_, pos) in query.iter(&self.legion) {
                cb_cnt += 1;
                let scr_pos = pos.0 - top_left;
                self.context.fill_rect(
                    scr_pos.x - cvars.g_cluster_bomb_size / 2.0,
                    scr_pos.y - cvars.g_cluster_bomb_size / 2.0,
                    cvars.g_cluster_bomb_size,
                    cvars.g_cluster_bomb_size,
                );
            }
            self.context.set_shadow_offset_x(0.0);
            self.context.set_shadow_offset_y(0.0);
            dbg_textd!(cb_cnt);
        }

        // Draw rockets
        self.context.set_stroke_style(&"white".into());
        let mut rocket_cnt = 0;
        let mut query = <(&Rocket, &Pos, &Vel)>::query();
        for (_, pos, vel) in query.iter(&self.legion) {
            rocket_cnt += 1;
            let scr_pos = pos.0 - top_left;
            if cvars.d_rockets_image {
                self.draw_img_center(&self.img_rocket, scr_pos, vel.0.to_angle())?;
            } else {
                self.context.begin_path();
                self.move_to(scr_pos);
                let scr_end = scr_pos - vel.0.normalized() * 16.0;
                self.line_to(scr_end);
                self.context.stroke();
            }
        }
        dbg_textd!(rocket_cnt);

        // Draw missile
        let gm = &self.gs.gm;
        let player_scr_pos = gm.pos - top_left;
        self.draw_img_center(&self.img_gm, player_scr_pos, gm.vel.to_angle())?;
        if cvars.d_debug_draw {
            self.context
                .fill_rect(player_scr_pos.x, player_scr_pos.y, 1.0, 1.0);
        }

        // Draw player vehicle chassis
        let tank = &self.gs.tank;
        let tank_scr_pos = tank.pos - top_left;
        self.draw_img_center(&self.imgs_vehicles[0], tank_scr_pos, tank.angle)?;
        if cvars.d_debug_draw {
            self.context.set_stroke_style(&"yellow".into());
            self.context.begin_path();
            let corners = Tank::corners(cvars, tank_scr_pos, tank.angle);
            self.move_to(corners[0]);
            self.line_to(corners[1]);
            self.line_to(corners[2]);
            self.line_to(corners[3]);
            self.context.close_path();
            self.context.stroke();
        }

        // TODO Draw cow

        // Draw player vehicle turret
        let turret_scr_pos = tank_scr_pos + tank.angle.to_vec2f() * cvars.r_turret_offset_tank;
        self.draw_img_center(&self.imgs_vehicles[1], turret_scr_pos, tank.angle)?;

        // Draw other tanks
        let mut query = <(&Pos, &Angle, &Hitbox)>::query();
        for (pos, angle, _hitbox) in query.iter(&self.legion) {
            let scr_pos = pos.0 - top_left;
            self.draw_img_center(&self.img_tank_red, scr_pos, angle.0)?;
            if cvars.d_debug_draw {
                self.context.set_stroke_style(&"yellow".into());
                self.context.begin_path();
                let corners = Tank::corners(cvars, scr_pos, angle.0);
                self.move_to(corners[0]);
                self.line_to(corners[1]);
                self.line_to(corners[2]);
                self.line_to(corners[3]);
                self.context.close_path();
                self.context.stroke();
            }
        }

        // Draw explosions
        let iter: Box<dyn Iterator<Item = &Explosion>> = if cvars.r_explosions_reverse {
            Box::new(self.gs.explosions.iter().rev())
        } else {
            Box::new(self.gs.explosions.iter())
        };
        for explosion in iter {
            // It looks like the original animation is made for 30 fps.
            // Single stepping a recording of the original RecWars explosion in blender:
            // 13 sprites, 31 frames - examples:
            //      2,2,3,1,3,3,2,3,2,2,3,2,3
            //      2,2,2,3,1,3,2,2,3,2,2,3,4
            // This code produces similar results,
            // though it might display a single sprite for 4 frames slightly more often.
            let progress = (self.gs.frame_time - explosion.start_time) / cvars.r_explosion_duration;
            // 13 sprites in the sheet, 100x100 pixels per sprite
            let frame = (progress * 13.0).floor();
            let offset = frame * 100.0;
            let scr_pos = explosion.pos - top_left;
            self.context
                .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    &self.img_explosion,
                    offset,
                    0.0,
                    100.0,
                    100.0,
                    scr_pos.x - 50.0 * explosion.scale,
                    scr_pos.y - 50.0 * explosion.scale,
                    100.0 * explosion.scale,
                    100.0 * explosion.scale,
                )?;
        }

        // Draw walls
        let mut r = top_left_index.y;
        let mut y = -bg_offset.y;
        while y < self.canvas_size.y {
            let mut c = top_left_index.x;
            let mut x = -bg_offset.x;
            while x < self.canvas_size.x {
                let tile = self.map.col_row(c, r);

                if self.map.surface_of(tile).kind == Kind::Wall {
                    let img = &self.imgs_textures[tile.surface_index];
                    self.draw_img_top_left(img, Vec2::new(x, y), tile.angle)?;
                }

                c += 1;
                x += TILE_SIZE;
            }
            r += 1;
            y += TILE_SIZE;
        }

        // Draw debug lines and crosses
        // TODO clear in logic frames
        if cvars.d_debug_draw {
            DEBUG_LINES.with(|lines| {
                let mut lines = lines.borrow_mut();
                for line in lines.iter() {
                    self.context.set_stroke_style(&line.color.into());
                    let scr_begin = line.begin - top_left;
                    let scr_end = line.end - top_left;
                    self.context.begin_path();
                    self.move_to(scr_begin);
                    self.line_to(scr_end);
                    self.context.stroke();
                }
                lines.clear();
            });
            DEBUG_CROSSES.with(|crosses| {
                let mut crosses = crosses.borrow_mut();
                for cross in crosses.iter() {
                    self.context.set_stroke_style(&cross.color.into());
                    let scr_point = cross.point - top_left;
                    let top_left = scr_point - Vec2f::new(-3.0, -3.0);
                    let bottom_right = scr_point - Vec2f::new(3.0, 3.0);
                    let top_right = scr_point - Vec2f::new(3.0, -3.0);
                    let bottom_left = scr_point - Vec2f::new(-3.0, 3.0);
                    self.context.begin_path();
                    self.move_to(top_left);
                    self.line_to(bottom_right);
                    self.move_to(top_right);
                    self.line_to(bottom_left);
                    self.context.stroke();
                }
                crosses.clear();
            });
        }

        // Draw HUD:

        // Homing missile indicator
        self.context.set_stroke_style(&"rgb(0, 255, 0)".into());
        let dash_len = cvars.hud_missile_indicator_dash_length.into();
        let dash_pattern = Array::of2(&dash_len, &dash_len);
        self.context.set_line_dash(&dash_pattern)?;
        self.context.begin_path();
        self.context.arc(
            tank_scr_pos.x,
            tank_scr_pos.y,
            cvars.hud_missile_indicator_radius,
            0.0,
            2.0 * PI,
        )?;
        self.context.move_to(tank_scr_pos.x, tank_scr_pos.y);
        let dir = (self.gs.gm.pos - self.gs.tank.pos).normalized();
        let end = tank_scr_pos + dir * cvars.hud_missile_indicator_radius;
        self.line_to(end);
        self.context.stroke();
        self.context.set_line_dash(&Array::new())?;

        // Hit points (goes from green to red)
        let hp = self.gs.tank.hp;
        // Might wanna use https://crates.io/crates/colorsys if I need more color operations.
        // Hit points to color (poor man's HSV):
        // 0.0 = red
        // 0.0..0.5 -> increase green channel
        // 0.5 = yellow
        // 0.5..1.0 -> decrease red channel
        // 1.0 = green
        let r = 1.0 - (hp.clamped(0.5, 1.0) - 0.5) * 2.0;
        let g = hp.clamped(0.0, 0.5) * 2.0;
        let rgb = format!("rgb({}, {}, 0)", r * 255.0, g * 255.0);
        self.context.set_fill_style(&rgb.into());
        self.context.fill_rect(
            cvars.hud_hp_x,
            cvars.hud_hp_y,
            cvars.hud_hp_width * hp,
            cvars.hud_hp_height,
        );

        // Ammo
        self.context.set_fill_style(&"yellow".into());
        let fraction = match self.gs.tank.ammos[self.gs.cur_weapon] {
            Ammo::Loaded(_, count) => {
                let max = cvars.g_weapon_reload_ammo(self.gs.cur_weapon);
                count as f64 / max as f64
            }
            Ammo::Reloading(start, end) => {
                let max_diff = end - start;
                let cur_diff = self.gs.frame_time - start;
                cur_diff / max_diff
            }
        };
        self.context.fill_rect(
            cvars.hud_ammo_x,
            cvars.hud_ammo_y,
            cvars.hud_ammo_width * fraction,
            cvars.hud_ammo_height,
        );

        // Weapon icon
        // The original shadows were part of the image but this is good enough for now.
        let shadow_rgba = format!("rgba(0, 0, 0, {})", cvars.hud_weapon_icon_shadow_alpha);
        self.context.set_shadow_color(&shadow_rgba);
        self.context
            .set_shadow_offset_x(cvars.hud_weapon_icon_shadow_x);
        self.context
            .set_shadow_offset_y(cvars.hud_weapon_icon_shadow_y);
        self.draw_img_center(
            &self.imgs_weapon_icons[self.gs.cur_weapon],
            Vec2f::new(cvars.hud_weapon_icon_x, cvars.hud_weapon_icon_y),
            0.0,
        )?;
        self.context.set_shadow_offset_x(0.0);
        self.context.set_shadow_offset_y(0.0);

        // let mut scr_pos = Vec2f::new(50.0, 50.0);
        // self.draw_img_top_left(&self.img_tank, scr_pos, 0.0f64.to_radians())?;
        // self.context.stroke_rect(scr_pos.x, scr_pos.y, 50.0, 30.0);
        // scr_pos.y += 100.0;
        // self.draw_img_top_left(&self.img_tank, scr_pos, 45.0f64.to_radians())?;
        // self.context.stroke_rect(scr_pos.x, scr_pos.y, 50.0, 30.0);
        // scr_pos.y += 100.0;
        // self.draw_img_top_left(&self.img_tank, scr_pos, 90.0f64.to_radians())?;
        // self.context.stroke_rect(scr_pos.x, scr_pos.y, 50.0, 30.0);

        // let mut scr_pos = Vec2f::new(150.0, 50.0);
        // self.draw_img_center(&self.img_tank, scr_pos, 0.0f64.to_radians())?;
        // self.context.stroke_rect(scr_pos.x, scr_pos.y, 50.0, 30.0);
        // scr_pos.y += 100.0;
        // self.draw_img_center(&self.img_tank, scr_pos, 45.0f64.to_radians())?;
        // self.context.stroke_rect(scr_pos.x, scr_pos.y, 50.0, 30.0);
        // scr_pos.y += 100.0;
        // self.draw_img_center(&self.img_tank, scr_pos, 90.0f64.to_radians())?;
        // self.context.stroke_rect(scr_pos.x, scr_pos.y, 50.0, 30.0);

        self.context.set_fill_style(&"red".into());

        // Draw perf info
        if !self.update_durations.is_empty() {
            let mut sum = 0.0;
            let mut max = 0.0;
            for &dur in &self.update_durations {
                sum += dur;
                if dur > max {
                    max = dur;
                }
            }

            self.context.fill_text(
                &format!(
                    "update avg: {:.1}, max: {:.1}",
                    sum / self.update_durations.len() as f64,
                    max
                ),
                self.canvas_size.x - 150.0,
                self.canvas_size.y - 60.0,
            )?;
        }
        if !self.draw_durations.is_empty() {
            let mut sum = 0.0;
            let mut max = 0.0;
            for &dur in &self.draw_durations {
                sum += dur;
                if dur > max {
                    max = dur;
                }
            }

            self.context.fill_text(
                &format!(
                    "draw avg: {:.1}, max: {:.1}",
                    sum / self.draw_durations.len() as f64,
                    max
                ),
                self.canvas_size.x - 150.0,
                self.canvas_size.y - 45.0,
            )?;
        }

        // Draw FPS
        // TODO this is wrong with d_speed
        let fps = if self.frame_times.is_empty() {
            0.0
        } else {
            let diff_time = self.frame_times.back().unwrap() - self.frame_times.front().unwrap();
            let diff_frames = self.frame_times.len() - 1;
            diff_frames as f64 / diff_time
        };
        self.context.fill_text(
            &format!("FPS: {:.1}", fps),
            self.canvas_size.x - 60.0,
            self.canvas_size.y - 15.0,
        )?;

        // Draw debug text
        let mut y = 20.0;
        DEBUG_TEXTS.with(|texts| {
            let mut texts = texts.borrow_mut();
            if cvars.d_debug_text {
                for line in texts.iter() {
                    self.context.fill_text(line, 20.0, y).unwrap();
                    y += cvars.d_debug_text_line_height;
                }
            }
            texts.clear();
        });

        let end = self.performance.now();
        if self.draw_durations.len() >= 60 {
            self.draw_durations.pop_front();
        }
        self.draw_durations.push_back(end - start);

        Ok(())
    }

    fn move_to(&self, point: Vec2f) {
        self.context.move_to(point.x, point.y);
    }

    fn line_to(&self, point: Vec2f) {
        self.context.line_to(point.x, point.y);
    }

    /// Place the image's *top-left corner* at `screen_pos`,
    /// rotate it clockwise around its center.
    fn draw_img_top_left(
        &self,
        img: &HtmlImageElement,
        screen_pos: Vec2f,
        angle: f64,
    ) -> Result<(), JsValue> {
        let half_size = Vec2::new(img.natural_width(), img.natural_height()).as_() / 2.0;
        self.context
            .translate(screen_pos.x + half_size.x, screen_pos.y + half_size.y)?;
        self.context.rotate(angle)?;

        // Now back off to the img's corner and draw it.
        // This can be done either by translating -half_size, then drawing at 0,0
        // or at once by drawing at -half_size which is perhaps marginally more efficient.
        self.context
            .draw_image_with_html_image_element(img, -half_size.x, -half_size.y)?;

        self.context.reset_transform()?;

        Ok(())
    }

    /// Place the image's *center* at `screen_pos`,
    /// rotate it clockwise around its center.
    fn draw_img_center(
        &self,
        img: &HtmlImageElement,
        screen_pos: Vec2f,
        angle: f64,
    ) -> Result<(), JsValue> {
        let half_size = Vec2::new(img.natural_width(), img.natural_height()).as_() / 2.0;
        self.context.translate(screen_pos.x, screen_pos.y)?;
        self.context.rotate(angle)?;

        // Now back off to the img's corner and draw it.
        // This can be done either by translating -half_size, then drawing at 0,0
        // or at once by drawing at -half_size which his perhaps marginally more efficient.
        self.context
            .draw_image_with_html_image_element(img, -half_size.x, -half_size.y)?;

        self.context.reset_transform()?;

        Ok(())
    }
}

//! Native and WASM versions using the macroquad engine.

use std::path::Path;

use cvars_console_macroquad::MacroquadConsole;
use macroquad::prelude::*;
use time::{format_description, OffsetDateTime};

use crate::{
    common::ServerTimings,
    debug::{self, DEBUG_SHAPES, DEBUG_TEXTS, DEBUG_TEXTS_WORLD},
    net::{self, Connection},
    prelude::*,
    timing::{Durations, Fps},
};

pub struct Client {
    pub assets: Assets,

    pub map: Map,

    pub gs: GameState,

    pub cg: ClientGame,

    /// Game time left over from previous update.
    pub dt_carry: f64,

    /// Time since the process started in seconds. Increases at wall clock speed even when paused.
    ///
    /// This is not meant to be used for anything that affects gameplay - use `gs.game_time` instead.
    pub real_time: f64,
    pub real_time_prev: f64,
    pub real_time_delta: f64,

    pub update_fps: Fps, // TODO remove?
    pub update_durations: Durations,
    pub gamelogic_fps: Fps,
    pub gamelogic_durations: Durations,
    pub screenshot_durations: Durations,
    /// Rendering consists of 2 steps:
    /// - Calling macroquad's draw functions
    /// - Calling next_frame() to actually render the frame
    pub render_fps: Fps,
    pub draw_calls_durations: Durations,
    pub engine_durations: Durations,

    /// Size of one player's view - either the whole screen or (a bit less than) half of it.
    pub viewport_size: Vec2f,
    pub client_mode: ClientMode,
    pub last_key: Option<KeyCode>,
    pub console: MacroquadConsole,
}

pub struct ClientGame {
    pub input1: ClientInput,
    pub input1_prev: ClientInput,
    pub input2: ClientInput,
    pub input2_prev: ClientInput,
    pub conn: Box<dyn Connection>,

    pub paused: bool,

    pub rail_beams: Vec<RailBeam>,
    pub explosions: Vec<Explosion>,

    pub notifications: Vec<Notification>,
    pub tmp_local_player_handle: Index, // LATER Proper splitscreen mode

    /// Last received server fps and durations info. Might be a few frames old.
    pub server_timings: ServerTimings,
}

#[derive(Debug)]
pub enum ClientMode {
    Singleplayer {
        player_handle: Index,
    },
    Splitscreen {
        render_targets: (RenderTarget, RenderTarget),
        player_handles: (Index, Index),
    },
}

impl Client {
    pub fn new(
        cvars: &Cvars,
        assets: Assets,
        map: Map,
        gs: GameState,
        conn: Box<dyn Connection>,
        player1_handle: Index,
        player2_handle: Option<Index>,
    ) -> Self {
        let cg = ClientGame {
            input1: ClientInput::empty(),
            input1_prev: ClientInput::empty(),
            input2: ClientInput::empty(),
            input2_prev: ClientInput::empty(),
            conn,

            paused: false,

            rail_beams: Vec::new(),
            explosions: Vec::new(),

            notifications: Vec::new(),
            tmp_local_player_handle: player1_handle,

            server_timings: ServerTimings::default(),
        };

        dbg_logf!("Window inner size: {}x{}", screen_width(), screen_height());
        let (viewport_size, client_mode) = if let Some(player2_handle) = player2_handle {
            let viewport_width = (screen_width() as f64 - cvars.r_splitscreen_gap) / 2.0;
            let viewport_size = Vec2f::new(viewport_width, screen_height() as f64);
            let viewport_left = render_target(viewport_size.x as u32, viewport_size.y as u32);
            let viewport_right = render_target(viewport_size.x as u32, viewport_size.y as u32);

            let client_mode = ClientMode::Splitscreen {
                render_targets: (viewport_left, viewport_right),
                player_handles: (player1_handle, player2_handle),
            };

            (viewport_size, client_mode)
        } else {
            let viewport_size = Vec2f::new(screen_width() as f64, screen_height() as f64);

            let client_mode = ClientMode::Singleplayer {
                player_handle: player1_handle,
            };

            (viewport_size, client_mode)
        };

        Self {
            assets,

            map,

            gs,

            cg,

            dt_carry: 0.0,
            real_time: 0.0,
            real_time_prev: 0.0,
            real_time_delta: 0.0,

            update_fps: Fps::new(),
            update_durations: Durations::new(),
            gamelogic_fps: Fps::new(),
            gamelogic_durations: Durations::new(),
            screenshot_durations: Durations::new(),
            render_fps: Fps::new(),
            draw_calls_durations: Durations::new(),
            engine_durations: Durations::new(),

            viewport_size,
            client_mode,
            last_key: None,
            console: MacroquadConsole::new(),
        }
    }

    pub fn ctx<'a>(&'a mut self, cvars: &'a Cvars) -> ClientFrameCtx<'a> {
        ClientFrameCtx::new(cvars, &self.map, &mut self.gs, &mut self.cg)
    }

    // LATER Stuff below is copied from server - merge?

    /// Run gamelogic frame(s) up to current time (in seconds).
    pub fn update(&mut self, cvars: &Cvars, real_time: f64) {
        // Recommended reading:
        // https://gafferongames.com/post/fix_your_timestep/
        // https://medium.com/@tglaiel/how-to-make-your-game-run-at-60fps-24c61210fe75

        self.update_fps.tick(cvars.d_fps_period, self.real_time);
        let start = macroquad::time::get_time();

        // Update time tracking variables
        self.real_time_prev = self.real_time;
        self.real_time = real_time;
        self.real_time_delta = self.real_time - self.real_time_prev;

        // We have to also send and receive outside gamelogic so pausing and unpausing works.
        let mut ctx = self.ctx(cvars);
        ctx.sys_net_send();
        ctx.sys_net_receive();
        if !self.cg.paused {
            let dt_update = self.real_time_delta * cvars.d_speed;
            self.gamelogic(cvars, dt_update);
        }

        let end = macroquad::time::get_time();
        self.update_durations
            .add(cvars.d_timing_samples, end - start);
    }

    /// The main game loop.
    fn gamelogic(&mut self, cvars: &Cvars, dt_update: f64) {
        // LATER Slow down time to prevent death spirals.
        // LATER Extrapolation (after client / server split).
        //  Gamecode should not know about it.
        //  Construct FrameData with the throwaway gs, gamelogic_tick_movement that only calls the movement systems?
        //  Don't accidentally call functions which modify state outside gs.

        if dt_update > 5.0 {
            dbg_logf!("WARNING: large dt_update: {dt_update}");
        }

        match cvars.sys_tickrate_mode {
            TickrateMode::Variable => {
                let game_time_target = self.gs.game_time + dt_update;
                self.gamelogic_tick(cvars, game_time_target);
            }
            TickrateMode::Fixed => {
                let dt = 1.0 / cvars.sys_tickrate_fixed_fps;
                let game_time_target = self.gs.game_time + self.dt_carry + dt_update;

                while self.gs.game_time + dt < game_time_target {
                    self.gamelogic_tick(cvars, self.gs.game_time + dt);
                }

                self.dt_carry = game_time_target - self.gs.game_time;
                if cvars.d_tickrate_fixed_carry {
                    dbg_logf!("Remaining time: {}", self.dt_carry);
                }
            }
        }
    }

    /// Run one frame of gamelogic.
    fn gamelogic_tick(&mut self, cvars: &Cvars, game_time: f64) {
        let start = macroquad::time::get_time();
        self.gamelogic_fps.tick(cvars.d_fps_period, self.real_time);

        // Update time tracking variables (in seconds)
        assert!(
            game_time >= self.gs.game_time,
            "game_time didn't increase: prev {}, current {}",
            self.gs.game_time,
            game_time,
        );
        self.gs.frame_num += 1;
        self.gs.game_time_prev = self.gs.game_time;
        self.gs.game_time = game_time;
        self.gs.dt = self.gs.game_time - self.gs.game_time_prev;
        debug::set_game_time(self.gs.game_time);

        if cvars.d_log_updates_cl {
            dbg_logf!(
                "gamelogic_tick f: {} gt: {:.03}",
                self.gs.frame_num,
                self.gs.game_time
            );
        }

        debug::clear_expired();

        dbg_textf!("{}", env!("GIT_VERSION"));
        dbg_textd!(self.gs.game_time);
        dbg_textd!(self.gs.game_time_prev);

        let mut ctx = self.ctx(cvars);

        ctx.sys_cleanup();

        ctx.sys_net_send();
        ctx.sys_net_receive();

        ctx.sys_debug_examples(v!(25 300));

        dbg_textf!("vehicle count: {}", self.gs.vehicles.len());
        dbg_textf!("projectile count: {}", self.gs.projectiles.len());
        dbg_textf!("explosion count: {}", self.cg.explosions.len());

        let end = macroquad::time::get_time();
        self.gamelogic_durations
            .add(cvars.d_timing_samples, end - start);
    }

    pub fn cl_input(&mut self, cvars: &Cvars) {
        if self.console.is_open() {
            // LATER Optionally set input to empty here.
            return;
        }

        if let Some(key_code) = get_last_key_pressed() {
            self.last_key = Some(key_code);
        }

        self.cg.input1_prev = self.cg.input1;
        self.cg.input1 = get_input1();
        self.cg.input2_prev = self.cg.input2;
        self.cg.input2 = get_input2();

        if !self.cg.input1_prev.pause && self.cg.input1.pause {
            let msg = ClientMessage::Pause;
            self.ctx(cvars).net_send(msg);
        }
    }

    pub fn post_render(&mut self, cvars: &Cvars) {
        if cvars.cl_screenshots {
            self.save_screenshot(cvars);
        }
    }

    fn save_screenshot(&mut self, cvars: &Cvars) {
        // Use tmpfs to avoid writing to disk:
        // sudo mount -o size=2G -t tmpfs none screenshots
        // Not sure if it's faster though.

        // get_screen_data leaks memory and is not released when quitting.
        // Use this to clear it:
        // sudo zsh -c 'echo 3 >| /proc/sys/vm/drop_caches'
        // https://github.com/not-fl3/macroquad/issues/655

        let t1 = get_time();

        let format =
            format_description::parse("[year]-[month]-[day]--[hour]-[minute]-[second]").unwrap();
        let dt = OffsetDateTime::now_utc().format(&format).unwrap();

        let path = cvars
            .cl_screenshot_path
            .replace("{date_time}", &dt)
            .replace("{frame_num}", &format!("{:06}", self.gs.frame_num))
            .replace("{game_time}", &format!("{:.03}", self.gs.game_time));

        let dir = Path::new(&path).parent().unwrap();
        // It takes 0.3 ms to create the directory on my machine,
        // after that it takes just 0.01 ms so it's ok to do every frame.
        std::fs::create_dir_all(&dir).unwrap();

        // get_screen_data() takes between 10 and 20 ms at 1600x900,
        // so it's not fast enough to record while playing
        // no matter how well we optimize the rest.
        let img = get_screen_data();

        // Macroquad offers `export_png` which is misnamed
        // because it supports any format based on suffix.
        // It flips the image upside down so it saves correctly
        // which costs another ~10 ms.
        // It also unconditionally panicks on failure.
        //
        // Encoding to TGA is the fastest but still costs another ~20 ms together with saving.

        image::save_buffer(
            &path,
            &img.bytes,
            screen_width() as u32,
            screen_height() as u32,
            image::ColorType::Rgba8,
        )
        .soft_unwrap();

        // LATER Render to render target instead of screen,
        //  save that to avoid get_screen_data().
        // LATER Consider 1) saving the raw data
        //  2) on another thread 3) without alpha channel.

        let t2 = get_time();
        self.screenshot_durations
            .add(cvars.d_timing_samples, t2 - t1);
    }
}

impl ClientFrameCtx<'_> {
    pub fn sys_cleanup(&mut self) {
        self.cg.rail_beams.retain(|beam| {
            beam.start_time + self.cvars.cl_railgun_trail_duration > self.gs.game_time
        });
        self.cg.explosions.retain(|explosion| {
            let age = self.gs.game_time - explosion.start_time;
            let progress = age / self.cvars.r_explosion_duration;
            progress <= 1.0
        });
        self.cg.notifications.retain(|notification| {
            self.gs.game_time - notification.start_time < self.cvars.hud_notifications_duration
        });
    }

    pub fn sys_net_send(&mut self) {
        // LATER Separate players

        let input = self.cg.input1.merged(self.cg.input2);

        let net_input = input.to_net_input();
        let msg = ClientMessage::Input(net_input);
        self.net_send(msg);
    }

    pub fn net_send(&mut self, msg: ClientMessage) {
        let net_msg = net::serialize(msg);
        let res = self.cg.conn.send(&net_msg);
        if let Err(e) = res {
            // LATER Not warning, don't exit, return to menu
            dbg_logf!("WARNING: Server disconnected: {}", e);
            std::process::exit(1);
        }
    }

    pub fn sys_net_receive(&mut self) {
        let (msgs, closed) = self.cg.conn.receive_sm();
        for msg in msgs {
            match msg {
                ServerMessage::Init(_) => {
                    dbg_logf!("WARNING: Server sent redundant init, ignoring")
                }
                ServerMessage::Update(update) => self.handle_update(update),

                ServerMessage::Paused(paused) => self.cg.paused = paused,

                ServerMessage::AddPlayer(init) => self.init_player(init),
                ServerMessage::SpawnVehicle(init) => self.init_vehicle(init),
                ServerMessage::SpawnProjectile(init) => self.init_projectile(init),
                ServerMessage::SpawnExplosion(init) => self.init_explosion(init),

                ServerMessage::RailBeam(mut beam) => {
                    beam.start_time = self.gs.game_time; // LATER don't sent start_time from server
                    self.cg.rail_beams.push(beam);
                }

                ServerMessage::RemovePlayer { index } => {
                    let player_handle = self.gs.players.slot_to_index(index).unwrap();
                    let name = self.gs.players[player_handle].name.clone();
                    self.remove_player(player_handle);
                    dbg_logf!("Player {name:?} removed");
                    // LATER Chat notification
                }
                ServerMessage::DestroyProjectile { index } => {
                    // LATER Explosion here instead of SpawnExplosion?
                    let old = self.gs.projectiles.remove_by_slot(index);
                    soft_assert!(old.is_some());
                }
                ServerMessage::Kill(kill) => self.handle_kill(kill),
            }
        }

        if closed {
            dbg_logf!("Server closed the connection, exiting");
        }
    }

    fn init_explosion(&mut self, init: ExplosionInit) {
        let ExplosionInit { pos, scale, bfg } = init;
        // LATER Setting start_time to client game_time means the animation plays from the start
        // but also that the explosion is delayed compared to the server. Is this what we want?
        let explosion = Explosion::new(pos, scale, self.gs.game_time, bfg);
        self.cg.explosions.push(explosion);
    }

    pub fn handle_update(&mut self, update: Update) {
        // Using destructuring here so we get an error if a field is added but not read.
        let Update {
            frame_num,
            game_time,
            game_time_prev: _, // LATER
            dt: _,             // LATER
            player_inputs,
            vehicles,
            projectiles,
            debug_texts,
            debug_texts_world,
            debug_shapes,
            server_timings,
        } = update;

        if self.cvars.d_log_updates_cl {
            dbg_logf!("handle_update f: {} gt: {:.03}", frame_num, game_time);
        }

        for InputUpdate { index, net_input } in player_inputs {
            let (_handle, player) = self.gs.players.get_by_slot_mut(index).unwrap();
            player.input_prev = player.input;
            player.input = net_input;
        }

        for VehicleUpdate {
            index,
            physics:
                EntityPhysics {
                    pos,
                    vel,
                    angle,
                    turn_rate,
                },
            turret_angle_current,
            turret_angle_wanted,
        } in vehicles
        {
            let (_handle, vehicle) = self.gs.vehicles.get_by_slot_mut(index).unwrap();
            vehicle.pos = pos;
            vehicle.vel = vel;
            vehicle.angle = angle;
            vehicle.turn_rate = turn_rate;
            vehicle.turret_angle_current = turret_angle_current;
            vehicle.turret_angle_wanted = turret_angle_wanted;
        }

        for ProjectileUpdate {
            index,
            physics:
                EntityPhysics {
                    pos,
                    vel,
                    angle,
                    turn_rate,
                },
        } in projectiles
        {
            let (_handle, projectile) = self.gs.projectiles.get_by_slot_mut(index).unwrap();
            projectile.pos = pos;
            projectile.vel = vel;
            projectile.angle = angle;
            projectile.turn_rate = turn_rate;
        }

        DEBUG_TEXTS.with(|texts| {
            let mut texts = texts.borrow_mut();
            texts.extend(debug_texts);
        });
        DEBUG_TEXTS_WORLD.with(|texts| {
            let mut texts = texts.borrow_mut();
            texts.extend(debug_texts_world);
        });
        DEBUG_SHAPES.with(|shapes| {
            let mut shapes = shapes.borrow_mut();
            shapes.extend(debug_shapes);
        });

        self.cg.server_timings = server_timings;
    }

    pub fn handle_kill(&mut self, kill: Kill) {
        let Kill { attacker, victim } = kill;

        // LATER Check client and server scores are the same at the end of match
        // LATER Merge with DestroyVehicle?

        let attacker_handle = self.gs.players.slot_to_index(attacker).unwrap();
        let victim_handle = self.gs.players.slot_to_index(victim).unwrap();

        let attacker = &mut self.gs.players[attacker_handle];
        if victim_handle == self.cg.tmp_local_player_handle && attacker_handle != victim_handle {
            self.cg.notifications.push(Notification::new(
                format!("You were killed by {}", attacker.name),
                self.cvars.hud_notifications_color_death,
                self.gs.game_time,
            ));
        }

        let victim = &mut self.gs.players[victim_handle];
        if attacker_handle == self.cg.tmp_local_player_handle {
            if attacker_handle == victim_handle {
                self.cg.notifications.push(Notification::new(
                    "You committed suicide".to_owned(),
                    self.cvars.hud_notifications_color_death,
                    self.gs.game_time,
                ));
            } else {
                self.cg.notifications.push(Notification::new(
                    format!("You killed {}", victim.name),
                    self.cvars.hud_notifications_color_kill,
                    self.gs.game_time,
                ));
            }
        }

        let vehicle = &mut self.gs.vehicles[victim.vehicle.unwrap()];
        vehicle.hp_fraction = 0.0;

        self.update_score_kill(attacker_handle, victim_handle);
    }
}

//! The authoritative server in a client-server game architecture - all data affecting gameplay, no networking yet.

use thunderdome::Index;

use crate::{
    cvars::{Cvars, TickrateMode},
    debugging,
    entities::{Ai, Player},
    game_state::{ArenaExt, GameState, Input},
    map::Map,
    systems,
    timing::{Durations, Fps, Time},
    BOT_NAMES,
};

#[derive(Debug)]
pub struct Server {
    pub(crate) time: Box<dyn Time>,
    pub map: Map,
    pub gs: GameState,
    /// Game time left over from previous update.
    pub(crate) dt_carry: f64,
    pub(crate) gs_fixed: GameState,
    /// Time since game started in seconds. Increases at wall clock speed even when paused.
    ///
    /// This is not meant to be used for anything that affects gameplay - use `gs.game_time` instead.
    pub(crate) real_time: f64,
    pub(crate) real_time_prev: f64,
    pub(crate) real_time_delta: f64,
    pub(crate) paused: bool,
    pub(crate) update_fps: Fps,
    pub(crate) update_durations: Durations,
    pub(crate) gamelogic_fps: Fps,
    pub(crate) gamelogic_durations: Durations,
}

impl Server {
    pub fn new(cvars: &Cvars, time: Box<dyn Time>, map: Map, mut gs: GameState) -> Self {
        let bots_count = map.spawns().len().min(cvars.bots_max);
        // TODO port dbg_* to macroquad
        // dbg_logf!(
        //     "Spawns per bot: {}",
        //     map.spawns().len() as f64 / bots_count as f64
        // );
        // dbg_logf!(
        //     "Tiles per bot: {}",
        //     (map.width() * map.height()) as f64 / bots_count as f64
        // );
        for i in 0..bots_count {
            let name = if i < BOT_NAMES.len() {
                BOT_NAMES[i].to_owned()
            } else {
                format!("Bot {}", i + 1)
            };
            let player = Player::new(name);
            let player_handle = gs.players.insert(player);
            gs.ais.insert(Ai::new(player_handle));
        }

        for handle in gs.players.iter_handles() {
            systems::spawn_vehicle(cvars, &mut gs, &map, handle, false);
        }

        Self {
            time,
            map,
            gs: gs.clone(),
            dt_carry: 0.0,
            gs_fixed: gs,
            real_time: 0.0,
            real_time_prev: 0.0,
            real_time_delta: 0.0,
            paused: false,
            update_fps: Fps::new(),
            update_durations: Durations::new(),
            gamelogic_fps: Fps::new(),
            gamelogic_durations: Durations::new(),
        }
    }

    pub fn input(&mut self, player_handle: Index, input: Input) {
        self.gs.inputs_prev.update(&self.gs.players);
        self.gs.players[player_handle].input = input;
        self.gs_fixed.inputs_prev.update(&self.gs_fixed.players);
        self.gs_fixed.players[player_handle].input = input;
    }

    /// Run gamelogic frame(s) up to current time (in seconds).
    pub fn update(&mut self, cvars: &Cvars, real_time: f64) {
        // Recommended reading: https://gafferongames.com/post/fix_your_timestep/

        // Update time tracking variables
        self.real_time_prev = self.real_time;
        self.real_time = real_time;
        self.real_time_delta = self.real_time - self.real_time_prev;

        for (handle, player) in self.gs.players.iter() {
            let input_prev = self.gs.inputs_prev.get(handle);
            if !input_prev.pause && player.input.pause {
                self.paused = !self.paused;
            }
        }
        if !self.paused {
            let dt_update = self.real_time_delta * cvars.d_speed;
            self.gamelogic(cvars, dt_update);
        }
    }

    fn gamelogic(&mut self, cvars: &Cvars, dt_update: f64) {
        // TODO prevent death spirals
        // LATER impl the other modes
        // TODO allow switching at runtime
        match cvars.sv_gamelogic_mode {
            TickrateMode::Synchronized => {
                let game_time_target = self.gs.game_time + dt_update;
                self.gamelogic_tick(cvars, game_time_target);
            }
            TickrateMode::Fixed => {
                let game_time_target = self.gs.game_time + self.dt_carry + dt_update;
                loop {
                    // gs.game_time is still the previous frame here
                    let remaining = game_time_target - self.gs.game_time;
                    let dt = 1.0 / cvars.sv_gamelogic_fixed_fps;
                    if remaining < dt {
                        self.dt_carry = remaining;
                        break;
                    }
                    self.gamelogic_tick(cvars, self.gs.game_time + dt);
                }
            }
            TickrateMode::FixedOrSmaller => {
                // FIXME Input is ignored or duplicated depending on fixed FPS
                // http://localhost:8000/web/?map=Atrium&bots_max=5&sv_gamelogic_mode=2&sv_gamelogic_fixed_fps=90

                let dt_fixed = self.gs.game_time - self.gs_fixed.game_time;
                let game_time_target = self.gs_fixed.game_time + dt_fixed + dt_update;
                self.gs = self.gs_fixed.clone();
                let mut remaining;
                loop {
                    // gs.game_time is still the previous frame here
                    remaining = game_time_target - self.gs.game_time;
                    let dt = 1.0 / cvars.sv_gamelogic_fixed_fps;
                    if remaining < dt {
                        self.gs_fixed = self.gs.clone();
                        break;
                    }
                    self.gamelogic_tick(cvars, self.gs.game_time + dt);
                }
                if cvars.d_dbg {
                    //dbg_logd!(remaining); FIXME
                }
                self.gamelogic_tick(cvars, self.gs.game_time + remaining);
                // TODO skip too small steps?
            }
        }
        // TODO don't use game_time here?
    }

    fn gamelogic_tick(&mut self, cvars: &Cvars, game_time: f64) {
        let start = self.time.now();
        self.gamelogic_fps.tick(cvars.d_fps_period, self.real_time);

        // Update time tracking variables (in seconds)
        assert!(
            game_time >= self.gs.game_time,
            "game_time didn't increase: prev {}, current {}",
            self.gs.game_time,
            game_time,
        );
        self.gs.game_time_prev = self.gs.game_time;
        self.gs.game_time = game_time;
        self.gs.dt = self.gs.game_time - self.gs.game_time_prev;

        debugging::cleanup();

        // FIXME
        // dbg_textf!("{}", env!("GIT_VERSION"));
        // dbg_textd!(self.gs.game_time);
        // dbg_textd!(self.gs.game_time_prev);

        systems::cleanup(cvars, &mut self.gs);

        systems::ai::ai(cvars, &mut self.gs);

        systems::respawning(cvars, &mut self.gs, &self.map);

        systems::player_logic(&mut self.gs);

        systems::vehicle_logic(cvars, &mut self.gs);

        // It's probably a good idea to shoot before movement so that when turning
        // the shot angle corresponds to the vehicle angle the player saw last frame.
        systems::shooting(cvars, &mut self.gs);

        systems::vehicle_movement(cvars, &mut self.gs, &self.map);

        systems::gm_turning(cvars, &mut self.gs);

        systems::projectiles(cvars, &mut self.gs, &self.map);

        systems::projectiles_timeout(cvars, &mut self.gs);

        systems::self_destruct(cvars, &mut self.gs);

        dbg_textf!("vehicle count: {}", self.gs.vehicles.len());
        dbg_textf!("projectile count: {}", self.gs.projectiles.len());
        dbg_textf!("explosion count: {}", self.gs.explosions.len());

        let end = self.time.now();
        self.gamelogic_durations
            .add(cvars.d_timing_samples, end - start);
    }
}

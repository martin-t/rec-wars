//! The authoritative server in a client-server game architecture - all data affecting gameplay, no networking yet.

use crate::{
    cvars::TickrateMode,
    debug,
    prelude::*,
    sys_ai, systems,
    timing::{Durations, Fps},
};

const BOT_NAMES: [&str; 20] = [
    "Dr. Dead",
    "Sir Hurt",
    "Mr. Pain",
    "PhD. Torture",
    "Mrs. Chestwound",
    "Ms. Dismember",
    "Don Lobotomy",
    "Lt. Dead",
    "Sgt. Dead",
    "Private Dead",
    "Colonel Dead",
    "Captain Dead",
    "Major Dead",
    "Commander Dead",
    "Díotóir",
    "Fireman",
    "Goldfinger",
    "Silverfinger",
    "Bronzefinger",
    "President Dead",
];

#[derive(Debug)]
pub struct Server {
    pub map: Map,
    pub gs: GameState,
    /// Game time left over from previous update.
    pub dt_carry: f64,
    pub gs_fixed: GameState,
    /// Time since game started in seconds. Increases at wall clock speed even when paused.
    ///
    /// This is not meant to be used for anything that affects gameplay - use `gs.game_time` instead.
    pub real_time: f64,
    pub real_time_prev: f64,
    pub real_time_delta: f64,
    pub paused: bool,
    pub update_fps: Fps,
    pub update_durations: Durations,
    pub gamelogic_fps: Fps,
    pub gamelogic_durations: Durations,
}

impl Server {
    pub fn new(cvars: &Cvars, map: Map) -> Self {
        let rng = SmallRng::seed_from_u64(cvars.d_seed);
        let mut gs = GameState::new(rng);

        let bots_count = map.spawns().len().min(cvars.bots_max);
        dbg_logf!(
            "Spawns per bot: {}",
            map.spawns().len() as f64 / bots_count as f64
        );
        dbg_logf!(
            "Tiles per bot: {}",
            (map.width() * map.height()) as f64 / bots_count as f64
        );
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

    pub fn connect(&mut self, cvars: &Cvars, name: &str) -> Index {
        let player = Player::new(name.to_owned());
        let player_handle = self.gs.players.insert(player.clone());
        let player_handle2 = self.gs_fixed.players.insert(player);
        assert_eq!(player_handle, player_handle2);
        systems::spawn_vehicle(cvars, &mut self.gs, &self.map, player_handle, true);
        player_handle
    }

    pub fn snapshot_inputs(&mut self) {
        self.gs.inputs_prev.snapshot(&self.gs.players);
        self.gs_fixed.inputs_prev.snapshot(&self.gs_fixed.players);
    }

    pub fn input(&mut self, local_player_handle: Index, input: Input) {
        // LATER Keep timestamps of input events. When splitting frame into multiple steps, update input each step.
        self.gs.players[local_player_handle].input = input;
        self.gs_fixed.players[local_player_handle].input = input;
    }

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

        // Handle pause outside gamelogic so it works properly.
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

        let end = macroquad::time::get_time();
        self.update_durations
            .add(cvars.d_timing_samples, end - start);
    }

    fn gamelogic(&mut self, cvars: &Cvars, dt_update: f64) {
        // TODO prevent death spirals
        // LATER impl the other modes
        // TODO allow switching at runtime
        match cvars.sv_tickrate_mode {
            TickrateMode::Synchronized => {
                let game_time_target = self.gs.game_time + dt_update;
                self.gamelogic_tick(cvars, game_time_target);
            }
            TickrateMode::Fixed => {
                let game_time_target = self.gs.game_time + self.dt_carry + dt_update;
                loop {
                    // gs.game_time is still the previous frame here
                    let remaining = game_time_target - self.gs.game_time;
                    let dt = 1.0 / cvars.sv_tickrate_fixed_fps;
                    if remaining < dt {
                        self.dt_carry = remaining;
                        break;
                    }
                    if cvars.d_tickrate_remaining {
                        dbg_logf!("Remaining time: {}", remaining);
                    }
                    self.gamelogic_tick(cvars, self.gs.game_time + dt);
                }
            }
            TickrateMode::FixedOrSmaller => {
                // TODO Input is ignored or duplicated depending on fixed FPS
                // http://localhost:8000/web/?map=Atrium&bots_max=5&sv_gamelogic_mode=2&sv_gamelogic_fixed_fps=90
                // TODO Related: gs_fixed should only be used here, the rest of the code shouldn't know about it.

                let dt_fixed = self.gs.game_time - self.gs_fixed.game_time;
                let game_time_target = self.gs_fixed.game_time + dt_fixed + dt_update;
                self.gs = self.gs_fixed.clone();
                let mut remaining;
                loop {
                    // gs.game_time is still the previous frame here
                    remaining = game_time_target - self.gs.game_time;
                    let dt = 1.0 / cvars.sv_tickrate_fixed_fps;
                    if remaining < dt {
                        self.gs_fixed = self.gs.clone();
                        break;
                    }
                    self.gamelogic_tick(cvars, self.gs.game_time + dt);
                }
                if cvars.d_tickrate_remaining {
                    dbg_logf!("Remaining time: {}", remaining);
                }
                self.gamelogic_tick(cvars, self.gs.game_time + remaining);
                // TODO skip too small steps?
            }
        }
        // TODO don't use game_time here?
    }

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
        self.gs.game_time_prev = self.gs.game_time;
        self.gs.game_time = game_time;
        self.gs.dt = self.gs.game_time - self.gs.game_time_prev;

        debug::clear_expired();

        dbg_textf!("{}", env!("GIT_VERSION"));
        dbg_textd!(self.gs.game_time);
        dbg_textd!(self.gs.game_time_prev);

        systems::cleanup(cvars, &mut self.gs);

        sys_ai::ai(cvars, &mut self.gs);

        systems::respawning(cvars, &mut self.gs, &self.map);

        systems::player_logic(&mut self.gs);

        systems::vehicle_logic(cvars, &mut self.gs);

        // It's probably a good idea to shoot before movement so that when turning
        // the shot angle corresponds to the vehicle angle the player saw last frame.
        systems::shooting(cvars, &mut self.gs);

        systems::vehicle_movement(cvars, &mut self.gs, &self.map);

        systems::hm_turning(cvars, &mut self.gs, &self.map);
        systems::gm_turning(cvars, &mut self.gs);

        systems::projectiles(cvars, &mut self.gs, &self.map);

        systems::projectiles_timeout(cvars, &mut self.gs);

        systems::self_destruct(cvars, &mut self.gs);

        systems::debug_examples(cvars);

        dbg_textf!("vehicle count: {}", self.gs.vehicles.len());
        dbg_textf!("projectile count: {}", self.gs.projectiles.len());
        dbg_textf!("explosion count: {}", self.gs.explosions.len());

        let end = macroquad::time::get_time();
        self.gamelogic_durations
            .add(cvars.d_timing_samples, end - start);
    }
}

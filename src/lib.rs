//! Initialization and game loop.

// Additional warnings that are allow by default (`rustc -W help`)
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unused)]
#![warn(clippy::all)]
#![allow(clippy::needless_range_loop)] // False positives

#[macro_use]
mod debugging; // keep first so the macros are available everywhere

mod cvars;
mod entities;
mod game_state;
mod map;
mod systems;
mod timing;

use std::fmt::Debug;

use game_state::ArenaExt;
use js_sys::Array;
use rand::prelude::*;
use thunderdome::Index;
use timing::{Durations, Fps};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement, Performance};

use crate::{
    cvars::{Cvars, TickrateMode},
    entities::{Ai, Player},
    game_state::{GameState, Input},
    map::Map,
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

#[wasm_bindgen]
#[derive(Debug)]
pub struct Game {
    client: Client,
    server: Server,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cvars: &Cvars,
        canvas: HtmlCanvasElement,
        context: CanvasRenderingContext2d,
        array_tiles: Array,
        array_vehicles: Array,
        array_wrecks: Array,
        array_weapon_icons: Array,
        img_rocket: HtmlImageElement,
        img_hm: HtmlImageElement,
        img_gm: HtmlImageElement,
        img_explosion: HtmlImageElement,
        img_explosion_cyan: HtmlImageElement,
        tex_list_text: &str,
        map_text: &str,
    ) -> Self {
        console_error_panic_hook::set_once();

        let rng = if cvars.d_seed == 0 {
            // This requires the `wasm-bindgen` feature on `rand` or it crashes at runtime.
            SmallRng::from_entropy()
        } else {
            SmallRng::seed_from_u64(cvars.d_seed)
        };

        let imgs_tiles = array_tiles
            .iter()
            .map(|tile| tile.dyn_into().unwrap())
            .collect();
        let imgs_vehicles = array_vehicles
            .iter()
            .map(|js_val| js_val.dyn_into().unwrap())
            .collect();
        let imgs_wrecks = array_wrecks
            .iter()
            .map(|js_val| js_val.dyn_into().unwrap())
            .collect();
        let imgs_weapon_icons = array_weapon_icons
            .iter()
            .map(|js_val| js_val.dyn_into().unwrap())
            .collect();

        let surfaces = map::load_tex_list(tex_list_text);
        let map = map::load_map(map_text, surfaces);

        let mut gs = GameState::new(rng);
        let name = "Player 1".to_owned();
        let player = Player::new(name);
        let player1_handle = gs.players.insert(player);

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
            client: Client {
                canvas,
                context,
                imgs_tiles,
                imgs_vehicles,
                imgs_wrecks,
                imgs_weapon_icons,
                img_rocket,
                img_hm,
                img_gm,
                img_explosion,
                img_explosion_cyan,
                render_fps: Fps::new(),
                render_durations: Durations::new(),
                player_handle: player1_handle,
            },
            server: Server {
                performance: web_sys::window().unwrap().performance().unwrap(),
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
            },
        }
    }

    /// Dump most of the game state to string.
    /// Can be used from the browser console as a very crude debugging tool: `game.to_debug_string()`.
    pub fn to_debug_string(&self) -> String {
        format!("{:#?}", self)
    }

    /// Process everything and render.
    /// `real_time` is in seconds.
    pub fn update_and_render(
        &mut self,
        real_time: f64,
        input: &Input,
        cvars: &Cvars,
    ) -> Result<(), JsValue> {
        // Handle input early so pause works immediately.
        // LATER Keep timestamps of input events. When splitting frame into multiple steps, update input each step.
        self.server.input(self.client.player_handle, *input);

        let start = self.server.performance.now();

        self.server
            .update_fps
            .tick(cvars.d_fps_period, self.server.real_time);
        self.server.update(cvars, real_time);

        let updated = self.server.performance.now();
        self.server
            .update_durations
            .add(cvars.d_timing_samples, updated - start);

        self.client
            .render_fps
            .tick(cvars.d_fps_period, self.server.real_time);
        systems::rendering::draw(self, cvars)?;

        let rendered = self.server.performance.now();
        self.client
            .render_durations
            .add(cvars.d_timing_samples, rendered - updated);

        Ok(())
    }

    pub fn visibility_change(&mut self, cvars: &Cvars, hidden: bool) {
        if hidden && cvars.sv_auto_pause_on_minimize {
            self.server.paused = true;
        } else if !hidden && cvars.sv_auto_unpause_on_restore {
            self.server.paused = false;
        }
    }
}

#[wasm_bindgen]
pub struct Client {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    imgs_tiles: Vec<HtmlImageElement>,
    imgs_vehicles: Vec<HtmlImageElement>,
    imgs_wrecks: Vec<HtmlImageElement>,
    imgs_weapon_icons: Vec<HtmlImageElement>,
    img_rocket: HtmlImageElement,
    img_hm: HtmlImageElement,
    img_gm: HtmlImageElement,
    img_explosion: HtmlImageElement,
    img_explosion_cyan: HtmlImageElement,
    render_fps: Fps,
    render_durations: Durations,
    player_handle: Index,
    // ^ When adding fields, consider adding them to Debug
}

impl Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Override the default Debug impl - The JS types don't print anything useful.
        f.debug_struct("Parts of Client")
            .field("render_fps", &self.render_fps)
            .field("render_durations", &self.render_durations)
            .field("player_handle", &self.player_handle)
            .finish()
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct Server {
    /// I want to track update and render time in Rust so i can draw the FPS counter and keep stats.
    /// Unfortunately, Instant::now() panics in WASM so i have to use performance.now().
    /// And just like in JS, it has limited precision in some browsers like firefox.
    performance: Performance,
    map: Map,
    gs: GameState,
    /// Game time left over from previous update.
    dt_carry: f64,
    gs_fixed: GameState,
    /// Time since game started in seconds. Increases at wall clock speed even when paused.
    ///
    /// This is not meant to be used for anything that affects gameplay - use `gs.game_time` instead.
    real_time: f64,
    real_time_prev: f64,
    real_time_delta: f64,
    paused: bool,
    update_fps: Fps,
    update_durations: Durations,
    gamelogic_fps: Fps,
    gamelogic_durations: Durations,
}

impl Server {
    fn input(&mut self, player_handle: Index, input: Input) {
        self.gs.inputs_prev.update(&self.gs.players);
        self.gs.players[player_handle].input = input;
        self.gs_fixed.inputs_prev.update(&self.gs_fixed.players);
        self.gs_fixed.players[player_handle].input = input;
    }

    /// Run gamelogic frame(s) up to current time (in seconds).
    fn update(&mut self, cvars: &Cvars, real_time: f64) {
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
                    dbg_logd!(remaining);
                }
                self.gamelogic_tick(cvars, self.gs.game_time + remaining);
                // TODO skip too small steps?
            }
        }
        // TODO don't use game_time here?
    }

    fn gamelogic_tick(&mut self, cvars: &Cvars, game_time: f64) {
        let start = self.performance.now();
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

        dbg_textf!("{}", env!("GIT_VERSION"));
        dbg_textd!(self.gs.game_time);
        dbg_textd!(self.gs.game_time_prev);

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

        let end = self.performance.now();
        self.gamelogic_durations
            .add(cvars.d_timing_samples, end - start);
    }
}

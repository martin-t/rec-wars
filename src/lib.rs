//! Initialization and game loop.

// Additional warnings that are allow by default (`rustc -W help`)
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unused)]
#![warn(clippy::all)]

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
use web_sys::{CanvasRenderingContext2d, HtmlImageElement, Performance};

use crate::{
    cvars::{Cvars, TickrateMode},
    entities::{Ai, Player},
    game_state::{GameState, Input},
    map::{Map, Vec2f},
};

#[wasm_bindgen]
#[derive(Debug)]
pub struct Game {
    /// I want to track update and render time in Rust so i can draw the FPS counter and keep stats.
    /// Unfortunately, Instant::now() panics in WASM so i have to use performance.now().
    /// And just like in JS, it has limited precision in some browsers like firefox.
    performance: Performance,
    client: Client,
    update_durations: Durations,
    draw_durations: Durations,
    server: Server,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cvars: &Cvars,
        context: CanvasRenderingContext2d,
        width: f64,
        height: f64,
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
        for i in 1..bots_count {
            let name = format!("Bot {}", i);
            let player = Player::new(name);
            let player_handle = gs.players.insert(player);
            gs.ais.insert(Ai::new(player_handle));
        }

        for handle in gs.players.iter_handles() {
            systems::spawn_vehicle(cvars, &mut gs, &map, handle, false);
        }

        Self {
            performance: web_sys::window().unwrap().performance().unwrap(),
            client: Client {
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
                canvas_size: Vec2f::new(width, height),
                fps: Fps::new(),
                player_handle: player1_handle,
            },
            update_durations: Durations::new(),
            draw_durations: Durations::new(),
            server: Server {
                map,
                gs,
                real_time: 0.0,
                real_time_prev: 0.0,
                real_time_delta: 0.0,
                paused: false,
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
        self.server.gs.inputs_prev.update(&self.server.gs.players);
        self.server.gs.players[self.client.player_handle].input = *input;

        self.client
            .fps
            .tick(cvars.d_fps_period, self.server.real_time);

        let start = self.performance.now();

        self.server.update(cvars, real_time);

        let updated = self.performance.now();
        self.update_durations
            .add(cvars.d_timing_samples, updated - start);

        systems::rendering::draw(self, cvars)?;

        let rendered = self.performance.now();
        self.draw_durations
            .add(cvars.d_timing_samples, rendered - updated);

        Ok(())
    }
}

#[wasm_bindgen]
pub struct Client {
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
    canvas_size: Vec2f,
    fps: Fps,
    player_handle: Index,
}

impl Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Override the default Debug impl - The JS types don't print anything useful.
        f.debug_struct("Parts of Client")
            .field("canvas_size", &self.canvas_size)
            .finish()
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct Server {
    map: Map,
    gs: GameState,
    /// Time since game started in seconds. Increases at wall clock speed even when paused.
    ///
    /// This is not meant to be used for anything that affects gameplay - use `gs.game_time` instead.
    real_time: f64,
    real_time_prev: f64,
    real_time_delta: f64,
    paused: bool,
}

impl Server {
    /// Run gamelogic frame(s) up to current time (in seconds).
    fn update(&mut self, cvars: &Cvars, real_time: f64) {
        // Recommended reading: https://gafferongames.com/post/fix_your_timestep/

        // Update time tracking variables
        self.real_time_prev = self.real_time;
        self.real_time = real_time;
        self.real_time_delta = self.real_time - self.real_time_prev;

        let diff_scaled = self.real_time_delta * cvars.d_speed;
        let game_time_target = self.gs.game_time + diff_scaled;

        for (handle, player) in self.gs.players.iter() {
            if player.input.pause && !self.gs.inputs_prev.get(handle).pause {
                self.paused = !self.paused;
            }
        }
        if !self.paused {
            self.gamelogic(cvars, game_time_target);
        }
    }

    fn gamelogic(&mut self, cvars: &Cvars, game_time_target: f64) {
        // TODO prevent death spirals
        // LATER impl the other modes
        match cvars.sv_gamelogic_mode {
            TickrateMode::Synchronized => {
                self.gamelogic_tick(cvars, game_time_target);
            }
            TickrateMode::SynchronizedBounded => unimplemented!(),
            TickrateMode::Fixed => loop {
                // gs.game_time is still the previous frame here
                let remaining = game_time_target - self.gs.game_time;
                let dt = 1.0 / cvars.sv_gamelogic_fixed_fps;
                if remaining < dt {
                    break;
                }
                self.gamelogic_tick(cvars, self.gs.game_time + dt);
            },
            TickrateMode::FixedOrSmaller => unimplemented!(),
        }
    }

    fn gamelogic_tick(&mut self, cvars: &Cvars, game_time: f64) {
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
    }
}

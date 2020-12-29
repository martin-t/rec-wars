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

use std::{collections::VecDeque, fmt::Debug};

use fnv::FnvHashMap;
use game_state::ArenaExt;
use js_sys::Array;
use rand::prelude::*;
use thunderdome::Arena;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlImageElement, Performance};

use crate::{
    cvars::{Cvars, TickrateMode},
    entities::{Ai, Player},
    game_state::{GameState, Input},
    map::{Map, Vec2f},
};

const STATS_FRAMES: usize = 60;

#[wasm_bindgen]
#[derive(Debug)]
pub struct Game {
    /// I want to track update and render time in Rust so i can draw the FPS counter and keep stats.
    /// Unfortunately, Instant::now() panics in WASM so i have to use performance.now().
    /// And just like in JS, it has limited precision in some browsers like firefox.
    performance: Performance,
    client: Client,
    /// Saved frame times in seconds over some period of time to measure FPS
    frame_times: VecDeque<f64>,
    update_durations: VecDeque<f64>,
    draw_durations: VecDeque<f64>,
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

        let mut players = Arena::new();

        let name = "Player 1".to_owned();
        let player = Player::new(name);
        let player1_handle = players.insert(player);

        let mut gs = GameState {
            rng,
            frame_time: 0.0,
            dt: 0.0,
            rail_beams: Vec::new(),
            rail_hits: FnvHashMap::default(),
            bfg_beams: Vec::new(),
            player_handle: player1_handle,
            explosions: Vec::new(),
            ais: Arena::new(),
            players,
            vehicles: Arena::new(),
            projectiles: Arena::new(),
        };
        let gs_prev = gs.clone();

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
                canvas_size: Vec2f::new(width, height),
                imgs_tiles,
                imgs_vehicles,
                imgs_wrecks,
                imgs_weapon_icons,
                img_rocket,
                img_hm,
                img_gm,
                img_explosion,
                img_explosion_cyan,
            },
            frame_times: VecDeque::new(),
            update_durations: VecDeque::new(),
            draw_durations: VecDeque::new(),
            server: Server { map, gs, gs_prev },
        }
    }

    /// Dump most of the game state to string.
    /// Can be used from the browser console as a very crude debugging tool: `game.to_debug_string()`.
    pub fn to_debug_string(&self) -> String {
        format!("{:#?}", self)
    }

    /// Run gamelogic up to `t` (in seconds) and render.
    pub fn update_and_render(
        &mut self,
        t: f64,
        input: &Input,
        cvars: &Cvars,
    ) -> Result<(), JsValue> {
        self.update(t, input, cvars);
        self.render(cvars)?;
        Ok(())
    }

    /// Run gamelogic frame.
    fn update(&mut self, t: f64, input: &Input, cvars: &Cvars) {
        // Recommended reading: https://gafferongames.com/post/fix_your_timestep/

        let start = self.performance.now();

        // TODO prevent death spirals
        match cvars.sv_gamelogic_mode {
            TickrateMode::Synchronized => {
                self.begin_frame(t);
                self.input(input);
                self.server.tick(cvars);
            }
            TickrateMode::SynchronizedBounded => todo!(),
            TickrateMode::Fixed => loop {
                // gs, not gs_prev, is the previous frame here
                let remaining = t - self.server.gs.frame_time;
                let dt = 1.0 / cvars.sv_gamelogic_fixed_fps;
                if remaining < dt {
                    break;
                }
                self.begin_frame(self.server.gs.frame_time + dt);
                self.input(input);
                self.server.tick(cvars);
            },
            TickrateMode::FixedOrSmaller => todo!(),
        }

        let end = self.performance.now();
        if self.update_durations.len() >= STATS_FRAMES {
            self.update_durations.pop_front();
        }
        self.update_durations.push_back(end - start);
    }

    /// Update time tracking variables (in seconds)
    fn begin_frame(&mut self, t: f64) {
        self.server.begin_frame(t);

        // There are multiple ways to count FPS.
        // Methods like using 1 / average_ms_per_frame end up with a lot of 59.9 vs 60.1 jitter.
        // Counting number of frames during the last second seems to give a stable 60.
        self.frame_times.push_back(t);
        while !self.frame_times.is_empty() && self.frame_times.front().unwrap() + 1.0 < t {
            self.frame_times.pop_front();
        }
    }

    fn input(&mut self, input: &Input) {
        self.server.gs.players[self.server.gs.player_handle].input = *input;
    }

    fn render(&mut self, cvars: &Cvars) -> Result<(), JsValue> {
        let start = self.performance.now();

        systems::rendering::draw(self, cvars)?;

        let end = self.performance.now();
        if self.draw_durations.len() >= STATS_FRAMES {
            self.draw_durations.pop_front();
        }
        self.draw_durations.push_back(end - start);

        Ok(())
    }
}

#[wasm_bindgen]
pub struct Client {
    context: CanvasRenderingContext2d,
    canvas_size: Vec2f,
    imgs_tiles: Vec<HtmlImageElement>,
    imgs_vehicles: Vec<HtmlImageElement>,
    imgs_wrecks: Vec<HtmlImageElement>,
    imgs_weapon_icons: Vec<HtmlImageElement>,
    img_rocket: HtmlImageElement,
    img_hm: HtmlImageElement,
    img_gm: HtmlImageElement,
    img_explosion: HtmlImageElement,
    img_explosion_cyan: HtmlImageElement,
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
    gs_prev: GameState,
}

impl Server {
    /// Update time tracking variables (in seconds)
    fn begin_frame(&mut self, t: f64) {
        self.gs_prev = self.gs.clone();
        self.gs.frame_time = t;
        self.gs.dt = self.gs.frame_time - self.gs_prev.frame_time;
        assert!(
            self.gs.frame_time >= self.gs_prev.frame_time,
            "frametime didn't increase: prev {}, current {}",
            self.gs_prev.frame_time,
            self.gs.frame_time,
        );
    }

    fn tick(&mut self, cvars: &Cvars) {
        dbg_textf!("{}", env!("GIT_VERSION"));

        systems::cleanup(cvars, &mut self.gs);

        systems::ai::ai(cvars, &mut self.gs);

        systems::respawning(cvars, &mut self.gs, &self.gs_prev, &self.map);

        systems::player_logic(&mut self.gs, &self.gs_prev);

        systems::vehicle_logic(cvars, &mut self.gs, &self.gs_prev);

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

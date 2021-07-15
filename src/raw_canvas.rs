//! The browser version using WASM and the canvas 2D API (without macroquad).

mod rendering;

use std::fmt::Debug;

use js_sys::Array;
use rand::prelude::*;
use thunderdome::Index;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

use crate::{
    cvars::Cvars,
    game_state::Input,
    map,
    server::Server,
    timing::{Durations, Fps, RawCanvasTime},
};

#[wasm_bindgen]
#[derive(Debug)]
pub struct RawCanvasGame {
    pub(crate) client: RawCanvasClient,
    pub(crate) server: Server,
}

#[wasm_bindgen]
impl RawCanvasGame {
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
            // Casting with `as` throws away some bits but it doesn't really matter,
            // better than using unsafe for transmute.
            // Another option would be SmallRng::from_entropy() but that requires enabling
            // some of rand's features *only* for raw_canvas because macroquad's WASM doesn't work with them.
            SmallRng::seed_from_u64(js_sys::Date::now() as u64)
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

        let time = Box::new(RawCanvasTime(
            web_sys::window().unwrap().performance().unwrap(),
        ));
        let mut server = Server::new(cvars, time, map, rng);

        let player1_handle = server.connect(cvars, "Player 1");

        Self {
            client: RawCanvasClient {
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
            server,
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

        self.server.update(cvars, real_time);

        self.client.render(&self.server, cvars)?;

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

pub(crate) struct RawCanvasClient {
    pub(crate) canvas: HtmlCanvasElement,
    pub(crate) context: CanvasRenderingContext2d,
    pub(crate) imgs_tiles: Vec<HtmlImageElement>,
    pub(crate) imgs_vehicles: Vec<HtmlImageElement>,
    pub(crate) imgs_wrecks: Vec<HtmlImageElement>,
    pub(crate) imgs_weapon_icons: Vec<HtmlImageElement>,
    pub(crate) img_rocket: HtmlImageElement,
    pub(crate) img_hm: HtmlImageElement,
    pub(crate) img_gm: HtmlImageElement,
    pub(crate) img_explosion: HtmlImageElement,
    pub(crate) img_explosion_cyan: HtmlImageElement,
    pub(crate) render_fps: Fps,
    pub(crate) render_durations: Durations,
    pub(crate) player_handle: Index,
    // ^ When adding fields, consider adding them to Debug
}

impl RawCanvasClient {
    fn render(&mut self, server: &Server, cvars: &Cvars) -> Result<(), JsValue> {
        self.render_fps.tick(cvars.d_fps_period, server.real_time);
        let start = server.time.now();

        rendering::draw(self, server, cvars)?;

        let end = server.time.now();
        self.render_durations
            .add(cvars.d_timing_samples, end - start);

        Ok(())
    }
}

impl Debug for RawCanvasClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Override the default Debug impl - The JS types don't print anything useful.
        f.debug_struct("Parts of Client")
            .field("render_fps", &self.render_fps)
            .field("render_durations", &self.render_durations)
            .field("player_handle", &self.player_handle)
            .finish()
    }
}

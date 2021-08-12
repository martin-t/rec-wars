//! Native and WASM versions using the macroquad engine.

mod rendering;

use futures::future;
use macroquad::prelude::*;
use thunderdome::Index;

use rec_wars::{
    cvars::Cvars,
    game_state::Input,
    map::Vec2f,
    server::Server,
    timing::{Durations, Fps},
};

#[derive(Debug)]
pub(crate) struct MacroquadClient {
    pub(crate) imgs_tiles: Vec<Texture2D>,
    pub(crate) imgs_vehicles: Vec<Texture2D>,
    pub(crate) imgs_wrecks: Vec<Texture2D>,
    pub(crate) imgs_weapon_icons: Vec<Texture2D>,
    pub(crate) img_rocket: Texture2D,
    pub(crate) img_hm: Texture2D,
    pub(crate) img_gm: Texture2D,
    pub(crate) img_explosion: Texture2D,
    pub(crate) img_explosion_cyan: Texture2D,
    pub(crate) render_fps: Fps,
    pub(crate) render_cmds_durations: Durations,
    pub(crate) rest_durations: Durations,
    pub(crate) viewport_size: Vec2f,
    pub(crate) client_mode: ClientMode,
    pub(crate) input_prev: Input,
    pub(crate) console: Console,
    pub(crate) console_visible: bool,
}

#[derive(Debug)]
pub(crate) enum ClientMode {
    Singleplayer {
        player_handle: Index,
    },
    Splitscreen {
        render_targets: (RenderTarget, RenderTarget),
        player_handles: (Index, Index),
    },
}

impl MacroquadClient {
    pub(crate) async fn new(
        cvars: &Cvars,
        player1_handle: Index,
        player2_handle: Option<Index>,
    ) -> Self {
        let loading_started = get_time();

        let paths_tiles = [
            "assets/tiles/g1.bmp",
            "assets/tiles/g2.bmp",
            "assets/tiles/g3.bmp",
            "assets/tiles/g_stripes.bmp",
            "assets/tiles/bunker1.bmp",
            "assets/tiles/ice1.bmp",
            "assets/tiles/ice.bmp",
            "assets/tiles/ice_side.bmp",
            "assets/tiles/ice_corner.bmp",
            "assets/tiles/g_spawn.bmp",
            "assets/tiles/road.bmp",
            "assets/tiles/water.bmp",
            "assets/tiles/snow.bmp",
            "assets/tiles/snow2.bmp",
            "assets/tiles/bunker2.bmp",
            "assets/tiles/base.bmp",
            "assets/tiles/water_side.bmp",
            "assets/tiles/water_corner.bmp",
            "assets/tiles/desert.bmp",
            "assets/tiles/d_rock.bmp",
            "assets/tiles/g2d.bmp",
            "assets/tiles/water_middle.bmp",
        ];
        let paths_vehicles = [
            "assets/vehicles/tank_chassis_flames.png",
            "assets/vehicles/tank_turret_flames.png",
            "assets/vehicles/hovercraft_chassis_flames.png",
            "assets/vehicles/hovercraft_turret_flames.png",
            "assets/vehicles/hummer_chassis_flames.png",
            "assets/vehicles/hummer_turret_flames.png",
        ];
        let paths_wrecks = [
            "assets/wrecks/tank.png",
            "assets/wrecks/hovercraft.png",
            "assets/wrecks/hummer.png",
        ];
        let paths_weapon_icons = [
            "assets/weapon_icons/mg.png",
            "assets/weapon_icons/rail.png",
            "assets/weapon_icons/cb.png",
            "assets/weapon_icons/rockets.png",
            "assets/weapon_icons/hm.png",
            "assets/weapon_icons/gm.png",
            "assets/weapon_icons/bfg.png",
        ];
        let paths_rest = [
            "assets/weapons/rocket.png",
            "assets/weapons/hm.png",
            "assets/weapons/gm.png",
            "assets/explosion.png",
            "assets/explosion_cyan.png",
        ];
        let paths = [
            &paths_tiles[..],
            &paths_vehicles[..],
            &paths_wrecks[..],
            &paths_weapon_icons[..],
            &paths_rest[..],
        ]
        .concat();

        let mut textures = future::try_join_all(paths.into_iter().map(|path| {
            draw_text("Loading...", 400.0, 400.0, 32.0, YELLOW);
            let tex = load_texture(path);
            draw_text("Loading...", 400.0, 400.0, 32.0, GREEN);
            tex
        }))
        .await
        .unwrap()
        .into_iter();
        draw_text("Loading...", 400.0, 400.0, 32.0, WHITE);

        let imgs_tiles = textures.by_ref().take(paths_tiles.len()).collect();
        let imgs_vehicles = textures.by_ref().take(paths_vehicles.len()).collect();
        let imgs_wrecks = textures.by_ref().take(paths_wrecks.len()).collect();
        let imgs_weapon_icons = textures.by_ref().take(paths_weapon_icons.len()).collect();
        let img_rocket = textures.next().unwrap();
        let img_hm = textures.next().unwrap();
        let img_gm = textures.next().unwrap();
        let img_explosion = textures.next().unwrap();
        let img_explosion_cyan = textures.next().unwrap();

        // LATER smoothing optional and configurable per image
        // LATER either use or remove r_smoothing (if raw_canvas is removed)
        img_explosion.set_filter(FilterMode::Nearest);
        img_explosion_cyan.set_filter(FilterMode::Nearest);

        let loading_done = get_time();
        rec_wars::dbg_logf!("Loaded assets in {:.2} s", loading_done - loading_started);

        rec_wars::dbg_logf!(
            "Detected screen size: {}x{}",
            screen_width(),
            screen_height()
        );
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
            render_cmds_durations: Durations::new(),
            rest_durations: Durations::new(),
            viewport_size,
            client_mode,
            input_prev: Input::new(),
            console: Console::new(),
            console_visible: false,
        }
    }

    pub(crate) fn process_input(&mut self, server: &mut Server) {
        let input1 = get_input1();
        let input2 = get_input2();

        if (input1.console && !self.input_prev.console)
            || (self.console_visible && input1.esc && !self.input_prev.esc)
        {
            self.console_visible = !self.console_visible;
        }
        self.input_prev = input1;

        match self.client_mode {
            ClientMode::Singleplayer { player_handle } => {
                let input = input1.merged(input2);
                server.input(player_handle, input);
            }
            ClientMode::Splitscreen {
                render_targets: _,
                player_handles: (player1_handle, player2_handle),
            } => {
                server.input(player1_handle, input1);
                server.input(player2_handle, input2);
            }
        }
    }

    pub(crate) fn render(&mut self, server: &Server, cvars: &Cvars) {
        self.render_fps.tick(cvars.d_fps_period, server.real_time);
        let start = get_time();

        rendering::render(self, server, cvars);

        let end = get_time();
        self.render_cmds_durations
            .add(cvars.d_timing_samples, end - start);
    }
}

#[derive(Debug)]
pub(crate) struct Console {
    prompt: String,
    history: Vec<String>,
}

impl Console {
    fn new() -> Self {
        Self {
            prompt: String::new(),
            history: Vec::new(),
        }
    }
}

// Keys to avoid in defaults:
//  - Ctrl - ctrl+W closes the browser tab
//  - Alt - shows/hides the firefox menu bar on linux
//  - Numpad - Some keyboards might not have it
//  - Keys that often depend on layout - https://github.com/not-fl3/macroquad/issues/260
// LATER Configurable input

fn get_input1() -> Input {
    let mut input = Input::new();
    if was_input_pressed(&[KeyCode::A]) {
        input.left = true;
    }
    if was_input_pressed(&[KeyCode::D]) {
        input.right = true;
    }
    if was_input_pressed(&[KeyCode::W]) {
        input.up = true;
    }
    if was_input_pressed(&[KeyCode::S]) {
        input.down = true;
    }
    if was_input_pressed(&[KeyCode::Q]) {
        input.turret_left = true;
    }
    if was_input_pressed(&[KeyCode::E]) {
        input.turret_right = true;
    }
    if was_input_pressed(&[KeyCode::V]) {
        input.prev_weapon = true;
    }
    if was_input_pressed(&[KeyCode::LeftShift, KeyCode::C]) {
        input.next_weapon = true;
    }
    if was_input_pressed(&[KeyCode::Space]) {
        input.fire = true;
    }
    if was_input_pressed(&[KeyCode::X]) {
        input.mine = true;
    }
    if was_input_pressed(&[KeyCode::G]) {
        input.self_destruct = true;
    }
    if was_input_pressed(&[KeyCode::R]) {
        input.horn = true;
    }
    if was_input_pressed(&[KeyCode::T]) {
        input.chat = true;
    }
    if was_input_pressed(&[KeyCode::Pause, KeyCode::P]) {
        input.pause = true;
    }
    if was_input_pressed(&[KeyCode::Semicolon]) {
        input.console = true;
    }
    if was_input_pressed(&[KeyCode::Escape]) {
        input.esc = true;
    }
    input
}

fn get_input2() -> Input {
    let mut input = Input::new();
    if was_input_pressed(&[KeyCode::Left]) {
        input.left = true;
    }
    if was_input_pressed(&[KeyCode::Right]) {
        input.right = true;
    }
    if was_input_pressed(&[KeyCode::Up]) {
        input.up = true;
    }
    if was_input_pressed(&[KeyCode::Down]) {
        input.down = true;
    }
    if was_input_pressed(&[KeyCode::Comma]) {
        input.turret_left = true;
    }
    if was_input_pressed(&[KeyCode::Period]) {
        input.turret_right = true;
    }
    if was_input_pressed(&[KeyCode::L]) {
        input.prev_weapon = true;
    }
    if was_input_pressed(&[
        KeyCode::Slash, // US layout
        KeyCode::Minus, // Same key, CZ layout
    ]) {
        input.next_weapon = true;
    }
    if was_input_pressed(&[KeyCode::RightShift]) {
        input.fire = true;
    }
    if was_input_pressed(&[KeyCode::M]) {
        input.mine = true;
    }
    if was_input_pressed(&[KeyCode::J]) {
        input.self_destruct = true;
    }
    if was_input_pressed(&[KeyCode::K]) {
        input.horn = true;
    }
    // No binds for shared actions like chat, pause, console and esc.
    // They're shared and defined on player 1.
    input
}

fn was_input_pressed(key_codes: &[KeyCode]) -> bool {
    for &key_code in key_codes {
        // Check both to avoid skipping input if it's pressed and released within one frame.
        if is_key_pressed(key_code) || is_key_down(key_code) {
            return true;
        }
    }
    false
}

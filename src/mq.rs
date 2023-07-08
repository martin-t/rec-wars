//! Native and WASM versions using the macroquad engine.

use cvars_console_macroquad::MacroquadConsole;
use macroquad::prelude::*;
use thunderdome::Index;

use crate::{
    cvars::Cvars,
    game_state::Input,
    map::Vec2f,
    server::Server,
    timing::{Durations, Fps},
};

#[derive(Debug)]
pub struct MacroquadClient {
    pub imgs_tiles: Vec<Texture2D>,
    pub imgs_vehicles: Vec<Texture2D>,
    pub imgs_wrecks: Vec<Texture2D>,
    pub imgs_weapon_icons: Vec<Texture2D>,
    pub img_rocket: Texture2D,
    pub img_hm: Texture2D,
    pub img_gm: Texture2D,
    pub img_explosion: Texture2D,
    pub img_explosion_cyan: Texture2D,
    pub render_fps: Fps,
    pub render_cmds_durations: Durations,
    pub rest_durations: Durations,
    pub viewport_size: Vec2f,
    pub client_mode: ClientMode,
    pub last_key: Option<KeyCode>,
    pub console: MacroquadConsole,
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

impl MacroquadClient {
    pub async fn new(cvars: &Cvars, player1_handle: Index, player2_handle: Option<Index>) -> Self {
        draw_text("Loading...", 400.0, 400.0, 32.0, GREEN);

        let loading_started = get_time();
        let mut cnt_bundled = 0;
        #[allow(unused_mut)] // WASM
        let mut cnt_loaded = 0;

        macro_rules! img {
            ($path:expr $(,)?) => {{
                let bytes = include_bytes!(concat!("../", $path));
                let bundled = Texture2D::from_file_with_format(bytes, None);

                // WASM:
                // Loading assets one by one is too slow in the browser because each is a separate request.
                // We can't use future::try_join_all because it crashes when compiled to WASM with the newest futures crate.
                // Might be because macroquad has its own special way of doing web-related things.
                // So just bundle the assets into the binary.
                #[cfg(target_arch = "wasm32")]
                {
                    cnt_bundled += 1;
                    bundled
                }

                // Desktop:
                // Load assets from disk so we can change them without recompiling.
                // Fall back to bundled assets if it fails.
                // This makes it possible to install the game from crates.io because it doesn't allow installing assets.
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let loaded = load_texture($path).await;
                    match loaded {
                        Ok(loaded) => {
                            cnt_loaded += 1;
                            loaded
                        }
                        Err(_) => {
                            cnt_bundled += 1;
                            bundled
                        }
                    }
                }
            }};
        }

        let imgs_tiles = vec![
            img!("assets/tiles/g1.bmp"),
            img!("assets/tiles/g2.bmp"),
            img!("assets/tiles/g3.bmp"),
            img!("assets/tiles/g_stripes.bmp"),
            img!("assets/tiles/bunker1.bmp"),
            img!("assets/tiles/ice1.bmp"),
            img!("assets/tiles/ice.bmp"),
            img!("assets/tiles/ice_side.bmp"),
            img!("assets/tiles/ice_corner.bmp"),
            img!("assets/tiles/g_spawn.bmp"),
            img!("assets/tiles/road.bmp"),
            img!("assets/tiles/water.bmp"),
            img!("assets/tiles/snow.bmp"),
            img!("assets/tiles/snow2.bmp"),
            img!("assets/tiles/bunker2.bmp"),
            img!("assets/tiles/base.bmp"),
            img!("assets/tiles/water_side.bmp"),
            img!("assets/tiles/water_corner.bmp"),
            img!("assets/tiles/desert.bmp"),
            img!("assets/tiles/d_rock.bmp"),
            img!("assets/tiles/g2d.bmp"),
            img!("assets/tiles/water_middle.bmp"),
        ];
        let imgs_vehicles = vec![
            img!("assets/vehicles/tank_chassis_flames.png"),
            img!("assets/vehicles/tank_turret_flames.png"),
            img!("assets/vehicles/hovercraft_chassis_flames.png"),
            img!("assets/vehicles/hovercraft_turret_flames.png"),
            img!("assets/vehicles/hummer_chassis_flames.png"),
            img!("assets/vehicles/hummer_turret_flames.png"),
        ];
        let imgs_wrecks = vec![
            img!("assets/wrecks/tank.png"),
            img!("assets/wrecks/hovercraft.png"),
            img!("assets/wrecks/hummer.png"),
        ];
        let imgs_weapon_icons = vec![
            img!("assets/weapon_icons/mg.png"),
            img!("assets/weapon_icons/rail.png"),
            img!("assets/weapon_icons/cb.png"),
            img!("assets/weapon_icons/rockets.png"),
            img!("assets/weapon_icons/hm.png"),
            img!("assets/weapon_icons/gm.png"),
            img!("assets/weapon_icons/bfg.png"),
        ];
        let img_rocket = img!("assets/weapons/rocket.png");
        let img_hm = img!("assets/weapons/hm.png");
        let img_gm = img!("assets/weapons/gm.png");
        let img_explosion = img!("assets/explosion.png");
        let img_explosion_cyan = img!("assets/explosion_cyan.png");

        let loading_done = get_time();
        let loading_duration = loading_done - loading_started;
        dbg_logf!("Loaded {} assets in {:.2} s", cnt_loaded, loading_duration);
        dbg_logf!("Using {} bundled assets as fallback", cnt_bundled);

        // LATER use r_smoothing (currently unused)
        // LATER smoothing optional and configurable per image
        // LATER allow changing smoothing at runtime
        img_explosion.set_filter(FilterMode::Nearest);
        img_explosion_cyan.set_filter(FilterMode::Nearest);

        dbg_logf!(
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
            last_key: None,
            console: MacroquadConsole::new(),
        }
    }

    pub fn process_input(&mut self, server: &mut Server) {
        if self.console.is_open() {
            return;
        }

        let input1 = get_input1();
        let input2 = get_input2();

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

        if let Some(key_code) = get_last_key_pressed() {
            self.last_key = Some(key_code);
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

    // The rest are shared actions defined on is player 1 only

    if was_input_pressed(&[KeyCode::Enter, KeyCode::T]) {
        input.chat = true;
    }
    if was_input_pressed(&[KeyCode::Pause, KeyCode::P]) {
        input.pause = true;
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
        KeyCode::Kp0,
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
    // They're defined on player 1.

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

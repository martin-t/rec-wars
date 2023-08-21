//! Native and WASM versions using the macroquad engine.

use std::path::Path;

use cvars_console_macroquad::MacroquadConsole;
use macroquad::prelude::*;

use crate::{
    prelude::*,
    server::Server,
    timing::{Durations, Fps},
};

#[derive(Debug)]
pub struct MacroquadClient {
    pub assets: Assets,
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
    pub fn new(
        cvars: &Cvars,
        assets: Assets,
        player1_handle: Index,
        player2_handle: Option<Index>,
    ) -> Self {
        // LATER use r_smoothing (currently unused)
        // LATER smoothing optional and configurable per image
        // LATER allow changing smoothing at runtime
        assets.tex_explosion.set_filter(FilterMode::Nearest);
        assets.tex_explosion_cyan.set_filter(FilterMode::Nearest);

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
                server.receive_input(player_handle, input);
            }
            ClientMode::Splitscreen {
                render_targets: _,
                player_handles: (player1_handle, player2_handle),
            } => {
                server.receive_input(player1_handle, input1);
                server.receive_input(player2_handle, input2);
            }
        }

        if let Some(key_code) = get_last_key_pressed() {
            self.last_key = Some(key_code);
        }
    }
}

#[derive(Debug)]
pub struct Assets {
    /// This is called texture list because the original ReCwar called it that.
    /// It's actually just a list of map surfaces, not all images/textures.
    pub texture_list: String,
    /// List of paths to maps supported by bots.
    pub bot_map_paths: Vec<String>,
    /// Map path -> map data as a string.
    pub maps: FnvHashMap<String, String>,
    /// Map name -> path.
    pub map_names_to_paths: FnvHashMap<String, String>,
    pub texs_tiles: Vec<Texture2D>,
    pub texs_vehicles: Vec<Texture2D>,
    pub texs_wrecks: Vec<Texture2D>,
    pub texs_weapon_icons: Vec<Texture2D>,
    pub tex_rocket: Texture2D,
    pub tex_hm: Texture2D,
    pub tex_gm: Texture2D,
    pub tex_explosion: Texture2D,
    pub tex_explosion_cyan: Texture2D,
}

impl Assets {
    pub async fn load_all() -> Self {
        let loading_started = get_time();

        let mut cnt_bundled = 0;
        #[allow(unused_mut)] // Unused on WASM
        let mut cnt_loaded = 0;

        macro_rules! asset {
            ($path:expr $(,)?) => {{
                let bundled = include_bytes!(concat!("../", $path)).to_vec();

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
                    let loaded = macroquad::file::load_file($path).await;
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

        let texture_list = String::from_utf8(asset!("data/texture_list.txt")).unwrap();

        let mut bot_map_paths = Vec::new();
        let mut maps = FnvHashMap::default();
        let mut map_names_to_paths = FnvHashMap::default();
        macro_rules! add_map {
            ($path:expr) => {{
                add_map_hidden!($path);

                bot_map_paths.push($path.to_owned());
            }};
        }
        macro_rules! add_map_hidden {
            ($path:expr) => {{
                let data = String::from_utf8(asset!($path)).unwrap();
                maps.insert($path.to_owned(), data);

                let name = Path::new($path).file_name().unwrap().to_str().unwrap();
                map_names_to_paths.insert(name.to_owned(), $path.to_owned());
            }};
        }
        // This is a subset of maps that are not blatantly broken with the current bots.
        // LATER Autodiscover maps without hardcoding.
        add_map_hidden!("maps/Arena.map");
        add_map_hidden!("maps/A simple plan (2).map");
        add_map!("maps/Atrium.map");
        add_map!("maps/Bunkers (2).map");
        add_map!("maps/Castle Islands (2).map");
        add_map!("maps/Castle Islands (4).map");
        add_map_hidden!("maps/Corners (4).map");
        add_map!("maps/Delta.map");
        add_map!("maps/Desert Eagle.map");
        add_map_hidden!("maps/Joust (2).map"); // Small map (narrow)
        add_map_hidden!("maps/Large front (2).map");
        add_map_hidden!("maps/Oases (4).map");
        add_map!("maps/Park.map");
        add_map!("maps/Roads.map");
        add_map!("maps/Snow.map");
        add_map!("maps/Spots (8).map");
        add_map_hidden!("maps/Vast Arena.map");
        add_map_hidden!("maps/extra/6 terrains (2).map");
        add_map_hidden!("maps/extra/A Cow Too Far.map");
        add_map_hidden!("maps/extra/All Water.map");
        add_map_hidden!("maps/extra/Battlegrounds (2).map");
        add_map_hidden!("maps/extra/Crossing.map"); // No spawns
        add_map!("maps/extra/Damned Rockets (2).map"); // Asymmetric CTF, left half like Castle Islands (2), right half has 2 bases
        add_map_hidden!("maps/extra/doom.map");
        add_map_hidden!("maps/extra/elements.map");
        add_map_hidden!("maps/extra/Exile (4).map"); // Small, many spawns
        add_map_hidden!("maps/extra/football.map");
        add_map!("maps/extra/Ice ring.map");
        add_map_hidden!("maps/extra/ice skating ring (2).map");
        add_map!("maps/extra/IceWorld.map");
        add_map!("maps/extra/I see you (2).map"); // Like Large Front (2) but without any cover
        add_map_hidden!("maps/extra/Knifflig (2).map");
        add_map_hidden!("maps/extra/Large.map");
        add_map_hidden!("maps/extra/Neutral.map");
        add_map!("maps/extra/Nile.map");
        add_map_hidden!("maps/extra/OK Corral (2).map"); // Tiny, not symmetric (upper spawn is closer)
        add_map_hidden!("maps/extra/Peninsulae (3).map");
        add_map_hidden!("maps/extra/River Crossings.map");
        add_map_hidden!("maps/extra/Road To Hell (2).map"); // Only 4 spawns in a tiny area
        add_map_hidden!("maps/extra/THE Crossing.map");
        add_map_hidden!("maps/extra/Thomap1 (4).map");
        add_map_hidden!("maps/extra/Town on Fire.map");
        add_map!("maps/extra/twisted (2).map");
        add_map_hidden!("maps/extra/winterhardcore.map");
        add_map!("maps/extra/Yellow and Green.map");
        add_map!("maps/extra2/Mini Islands (4).map");
        add_map_hidden!("maps/extra2/Symmetric.map");
        add_map_hidden!("maps/extra2/Training room.map");
        add_map_hidden!("maps/extra2/Winter (4).map");
        add_map_hidden!("maps/extra2/World War (2).map");

        macro_rules! tex {
            ($path:expr $(,)?) => {
                Texture2D::from_file_with_format(&asset!($path), None)
            };
        }
        let texs_tiles = vec![
            tex!("data/tiles/g1.bmp"),
            tex!("data/tiles/g2.bmp"),
            tex!("data/tiles/g3.bmp"),
            tex!("data/tiles/g_stripes.bmp"),
            tex!("data/tiles/bunker1.bmp"),
            tex!("data/tiles/ice1.bmp"),
            tex!("data/tiles/ice.bmp"),
            tex!("data/tiles/ice_side.bmp"),
            tex!("data/tiles/ice_corner.bmp"),
            tex!("data/tiles/g_spawn.bmp"),
            tex!("data/tiles/road.bmp"),
            tex!("data/tiles/water.bmp"),
            tex!("data/tiles/snow.bmp"),
            tex!("data/tiles/snow2.bmp"),
            tex!("data/tiles/bunker2.bmp"),
            tex!("data/tiles/base.bmp"),
            tex!("data/tiles/water_side.bmp"),
            tex!("data/tiles/water_corner.bmp"),
            tex!("data/tiles/desert.bmp"),
            tex!("data/tiles/d_rock.bmp"),
            tex!("data/tiles/g2d.bmp"),
            tex!("data/tiles/water_middle.bmp"),
        ];
        let texs_vehicles = vec![
            tex!("data/vehicles/tank_chassis_flames.png"),
            tex!("data/vehicles/tank_turret_flames.png"),
            tex!("data/vehicles/hovercraft_chassis_flames.png"),
            tex!("data/vehicles/hovercraft_turret_flames.png"),
            tex!("data/vehicles/hummer_chassis_flames.png"),
            tex!("data/vehicles/hummer_turret_flames.png"),
        ];
        let texs_wrecks = vec![
            tex!("data/wrecks/tank.png"),
            tex!("data/wrecks/hovercraft.png"),
            tex!("data/wrecks/hummer.png"),
        ];
        let texs_weapon_icons = vec![
            tex!("data/weapon_icons/mg.png"),
            tex!("data/weapon_icons/rail.png"),
            tex!("data/weapon_icons/cb.png"),
            tex!("data/weapon_icons/rockets.png"),
            tex!("data/weapon_icons/hm.png"),
            tex!("data/weapon_icons/gm.png"),
            tex!("data/weapon_icons/bfg.png"),
        ];
        let tex_rocket = tex!("data/weapons/rocket.png");
        let tex_hm = tex!("data/weapons/hm.png");
        let tex_gm = tex!("data/weapons/gm.png");
        let tex_explosion = tex!("data/explosion.png");
        let tex_explosion_cyan = tex!("data/explosion_cyan.png");

        let loading_done = get_time();
        let loading_duration = loading_done - loading_started;
        dbg_logf!("Loaded {} assets in {:.2} s", cnt_loaded, loading_duration);
        dbg_logf!("Using {} bundled assets as fallback", cnt_bundled);

        Self {
            texture_list,
            bot_map_paths,
            maps,
            map_names_to_paths,
            texs_tiles,
            texs_vehicles,
            texs_wrecks,
            texs_weapon_icons,
            tex_rocket,
            tex_hm,
            tex_gm,
            tex_explosion,
            tex_explosion_cyan,
        }
    }

    pub fn select_map<'a>(&'a self, cvars: &'a mut Cvars) -> &'a str {
        if cvars.g_map.is_empty() {
            // Pick a random map supported by bots.
            let index = cvars.d_seed as usize % self.bot_map_paths.len();
            let path = &self.bot_map_paths[index];
            cvars.g_map = path.clone();
            path
        } else if cvars.g_map.starts_with("maps/") {
            // Load the exact path.
            &cvars.g_map
        } else {
            // Attempt to find a map whose name starts with the given string.
            let mut matching = Vec::new();
            for (name, path) in &self.map_names_to_paths {
                if name.starts_with(&cvars.g_map) {
                    matching.push(path);
                }
            }
            if matching.is_empty() {
                panic!("ERROR: No maps found matching {}", cvars.g_map);
            } else if matching.len() > 1 {
                dbg_logf!("WARNING: Multiple maps found matching {}:", cvars.g_map);
                for path in &matching {
                    dbg_logf!("    {}", path);
                }
                matching[0]
            } else {
                matching[0]
            }
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

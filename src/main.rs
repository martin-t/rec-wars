// Additional warnings that are allow by default (`rustc -W help`)
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unused)]
#![warn(clippy::all)]
#![allow(clippy::needless_range_loop)] // False positives
#![allow(clippy::too_many_arguments)]

#[macro_use]
pub mod debug; // keep first so the macros are available everywhere

pub mod cvars;
pub mod entities;
pub mod game_state;
pub mod map;
pub mod mq;
pub mod prelude;
pub mod rendering;
pub mod server;
pub mod sys_ai;
pub mod systems;
pub mod timing;

use macroquad::prelude::*;

use crate::{mq::MacroquadClient, prelude::*, server::Server};

// LATER server/client/local mode
#[derive(Debug)]
struct Opts {
    /// Set cvar values - use key value pairs (separated by space).
    /// Example: g_armor 150 hud_names false
    cvars: Vec<String>,
}

fn get_opts() -> Opts {
    // Currently not using clap to save 150 ms on incremental rebuilds.
    Opts {
        cvars: std::env::args().skip(1).collect(),
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "RecWars".to_owned(),
        // In older macroquad, setting width and height to the size of the screen or larger
        // created a maximized window. This no longer works and the bottom OS panel covers
        // the window so this is an ugly compromise that should be good enough for most people.
        window_width: 1600,
        window_height: 900,
        // LATER Allow resizing - AFAICT there was an issue with infinite memory growth when resizing the render target.
        // Can't use `fullscreen: true` because of https://github.com/not-fl3/macroquad/issues/237.
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    draw_text("Loading...", 400.0, 400.0, 32.0, RED);
    show_mouse(false);

    let opts = get_opts();

    debug::set_endpoint("lo");

    let mut cvars = Cvars::new_rec_wars();

    #[cfg(feature = "web_splitscreen")]
    {
        cvars.cl_splitscreen = true;
    }

    let mut cvars_iter = opts.cvars.iter();
    while let Some(cvar_name) = cvars_iter.next() {
        let str_value = cvars_iter.next().unwrap();
        cvars.set_str(cvar_name, str_value).unwrap();
        dbg_logf!("{} = {}", cvar_name, cvars.get_string(cvar_name).unwrap());
    }

    if cvars.d_seed == 0 {
        let time_seed = macroquad::miniquad::date::now();
        cvars.d_seed = time_seed.to_bits();
    }
    dbg_logf!("Seed: {}", cvars.d_seed);

    let assets = Assets::load_all().await;

    let map_path = if cvars.g_map.is_empty() {
        // Pick a random map supported by bots.
        let index = cvars.d_seed as usize % assets.bot_map_paths.len();
        let path = &assets.bot_map_paths[index];
        cvars.g_map = path.clone();
        path
    } else if cvars.g_map.starts_with("maps/") {
        // Load the exact path.
        &cvars.g_map
    } else {
        // Attempt to find a map whose name starts with the given string.
        let mut matching = Vec::new();
        for (name, path) in &assets.map_names_to_paths {
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
    };
    dbg_logf!("Map: {}", map_path);

    let map_text = assets.maps.get(map_path).unwrap();
    let surfaces = map::parse_texture_list(&assets.texture_list);
    let map = map::parse_map(map_text, surfaces);

    let mut server = Server::new(&cvars, map);

    let player1_handle = server.connect(&cvars, "Player 1");
    let player2_handle = if cvars.cl_splitscreen {
        Some(server.connect(&cvars, "Player 2"))
    } else {
        None
    };
    let mut client = MacroquadClient::new(&cvars, assets, player1_handle, player2_handle);

    loop {
        let real_time = get_time();

        server.snapshot_inputs();

        client.process_input(&mut server);

        server.update(&cvars, real_time);

        rendering::render(&cvars, &mut client, &server);
        client.console.update(&mut cvars);

        let before = get_time();
        next_frame().await;
        let after = get_time();
        client
            .rest_durations
            .add(cvars.d_timing_samples, after - before);
    }
}

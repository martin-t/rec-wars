// Additional warnings that are allow by default (`rustc -W help`)
#![warn(missing_debug_implementations)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unused)]
#![warn(clippy::all)]
#![allow(clippy::needless_range_loop)] // False positives
#![allow(clippy::too_many_arguments)]

#[macro_use]
pub mod debugging; // keep first so the macros are available everywhere

pub mod cvars;
pub mod entities;
pub mod game_state;
pub mod map;
pub mod mq;
pub mod rendering;
pub mod server;
pub mod sys_ai;
pub mod systems;
pub mod timing;

use std::str;

use clap::Parser;
use macroquad::prelude::*;

use crate::{cvars::Cvars, mq::MacroquadClient, server::Server};

#[derive(Debug, Parser)]
struct Opts {
    /// 2 player local multiplayer
    #[arg(long)]
    splitscreen: bool,

    /// Set the map to play on (instead of random)
    #[arg(long)]
    map: Option<String>,

    /// Set cvar values - use key value pairs (separated by space).
    /// Example: g_armor 150 hud_names false
    cvars: Vec<String>,
}

fn get_opts() -> Opts {
    #[allow(unused_mut)]
    let mut opts = Opts::parse();

    // This is so I can easily toggle between compiling
    // singleplayer and splitscreen for the web.
    #[cfg(feature = "web_splitscreen")]
    {
        opts.splitscreen = true;
    }

    opts
}

fn window_conf() -> Conf {
    Conf {
        window_title: "RecWars".to_owned(),
        // In older macroquad, setting width and height to the size of the screen or larger
        // created a maximized window. This no longer works and the bottom OS panel covers
        // the window so this is an ugly compromise that should be good enough for most people.
        window_width: 1280,
        window_height: 1024,
        // LATER Allow resizing - AFAICT there was an issue with infinite memory growth when resizing the render target.
        // Can't use `fullscreen: true` because of https://github.com/not-fl3/macroquad/issues/237.
        window_resizable: false,
        ..Default::default()
    }
}

use macroquad::prelude::*;

#[macroquad::main("test")]
async fn main() {
    loop {
        info!("AAAAAAAAAAAAAAAAA");
        draw_text("MAIN", 400.0, 400.0, 32.0, RED);
        next_frame().await;
    }
}

//#[macroquad::main(window_conf)]
async fn main2() {
    let opts = get_opts();

    // This is a hack.
    // It seems that in the browser, MQ redraws the screen several times between here and the main loop
    // (even though there are no next_frame().await calls) so this doesn't stay up for long.
    // Let's just redraw it a few times during the loading process so the player sees something is happening.
    dbg_logf!("Loading...");
    draw_text("Loading...", 400.0, 400.0, 32.0, RED);

    show_mouse(false);

    dbg_logf!("Loading cvars");
    let mut cvars = Cvars::new_rec_wars();
    let mut cvars_iter = opts.cvars.iter();
    while let Some(cvar_name) = cvars_iter.next() {
        let str_value = cvars_iter.next().unwrap();
        cvars.set_str(cvar_name, str_value).unwrap();
        dbg_logf!("{} = {}", cvar_name, cvars.get_string(cvar_name).unwrap());
    }

    let time_seed = macroquad::miniquad::date::now();
    if cvars.d_seed == 0 {
        cvars.d_seed = time_seed.to_bits();
    }
    dbg_logf!("Seed: {}", cvars.d_seed);

    // LATER Load texture list and map in parallel with other assets
    let tex_list_bytes = load_file("assets/texture_list.txt").await.unwrap();
    draw_text("Loading...", 400.0, 400.0, 32.0, PURPLE);
    let tex_list_text = str::from_utf8(&tex_list_bytes).unwrap();
    let surfaces = map::load_tex_list(tex_list_text);

    // This is a subset of maps that are not blatantly broken with the current bots.
    let maps = [
        //"Arena",
        //"A simple plan (2)",
        "Atrium",
        "Bunkers (2)",
        "Castle Islands (2)",
        "Castle Islands (4)",
        //"Corners (4)",
        "Delta",
        "Desert Eagle",
        //"Joust (2)", // Small map (narrow)
        //"Large front (2)",
        //"Oases (4)",
        "Park",
        "Roads",
        "Snow",
        "Spots (8)",
        //"Vast Arena",
        //"extra/6 terrains (2)",
        //"extra/A Cow Too Far",
        //"extra/All Water",
        //"extra/Battlegrounds (2)",
        //"extra/Crossing", // No spawns
        "extra/Damned Rockets (2)", // Asymmetric CTF, left half like Castly Islands (2), right half has 2 bases
        //"extra/doom",
        //"extra/elements",
        //"extra/Exile (4)", // Tiny, many spawns
        //"extra/football",
        "extra/Ice ring",
        //"extra/ice skating ring (2)",
        "extra/IceWorld",
        "extra/I see you (2)", // Like Large Front (2) but without any cover
        //"extra/Knifflig (2)",
        //"extra/Large",
        //"extra/Neutral",
        "extra/Nile",
        //"extra/OK Corral (2)", // Small map, not symmetric (upper spawn is closer)
        //"extra/Peninsulae (3)",
        //"extra/River Crossings",
        //"extra/Road To Hell (2)", // Only 4 spawns in a tiny area
        //"extra/THE Crossing",
        //"extra/Thomap1 (4)",
        //"extra/Town on Fire",
        "extra/twisted (2)",
        //"extra/winterhardcore",
        "extra/Yellow and Green",
        "extra2/Mini Islands (4)",
        //"extra2/Symmetric",
        //"extra2/Training room",
        //"extra2/Winter (4)",
        //"extra2/World War (2)",
    ];
    let mut map_path = opts.map.unwrap_or_else(|| {
        // Intentionally not using cvars.d_seed here
        // so that setting the seed doesn't force a specific map.
        let index = time_seed as usize % maps.len();
        maps[index].to_owned()
    });
    if !map_path.ends_with(".map") {
        map_path.push_str(".map");
    }
    if !map_path.starts_with("maps/") {
        map_path.insert_str(0, "maps/");
    }
    dbg_logf!("Map: {}", map_path);

    let map_bytes = load_file(&map_path).await.unwrap();
    draw_text("Loading...", 400.0, 400.0, 32.0, PURPLE);
    let map_text = str::from_utf8(&map_bytes).unwrap();
    let map = map::load_map(map_text, surfaces);

    let mut server = Server::new(&cvars, map);

    let player1_handle = server.connect(&cvars, "Player 1");
    let player2_handle = if opts.splitscreen {
        Some(server.connect(&cvars, "Player 2"))
    } else {
        None
    };
    // LATER It can take some time for assets to load but the game is already running on the server.
    //       Load assets first, then connect.
    let mut client = MacroquadClient::new(&cvars, player1_handle, player2_handle).await;
    draw_text("Loading...", 400.0, 400.0, 32.0, PURPLE);

    loop {
        let real_time = get_time();

        server.snapshot_inputs();

        client.process_input(&mut server);

        server.update(&cvars, real_time);

        rendering::render(&mut client, &server, &cvars);
        client.console.update(&mut cvars);

        let before = get_time();
        next_frame().await;
        let after = get_time();
        client
            .rest_durations
            .add(cvars.d_timing_samples, after - before);
    }
}

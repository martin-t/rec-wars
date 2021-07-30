//! Entry point for the macroquad clients (both native and WASM).
//! The raw canvas + WASM client lives in lib.rs.

mod mq;

use std::{str, time::UNIX_EPOCH};

use macroquad::prelude::*;
use structopt::StructOpt;

use rec_wars::{cvars::Cvars, map, server::Server, timing::MacroquadTime};

use crate::mq::MacroquadClient;

#[derive(StructOpt, Debug)]
struct Opts {
    #[structopt(long)]
    splitscreen: bool,

    #[structopt(long, default_value = "maps/Atrium.map")]
    map: String,

    #[structopt(long)]
    seed: Option<u64>,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "RecWars".to_owned(),
        // Setting width and height to the size of the screen or larger
        // creates a maximized window. Tested on Kubuntu 20.10.
        // Not using larger values (or i32::MAX) in case other platforms behave differently.
        window_width: 1920,
        window_height: 1080,
        // LATER Prevent resizing or handle it properly when using render targets.
        // Can't use fullscreen: true because of https://github.com/not-fl3/macroquad/issues/237.
        // Can't use window_resizable: false because Kubuntu's panel would cover the bottom part of the window.
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let opts = Opts::from_args();

    // This is a hack.
    // It seems that in the browser, MQ redraws the screen several times between here and the main loop
    // (even though there are no next_frame().await calls) so this doesn't stay up for long.
    // Let's just redraw it a few times during the loading process so the player sees something is happening.
    draw_text("Loading...", 400.0, 400.0, 32.0, RED);

    let mut cvars = Cvars::new_rec_wars();
    if let Some(seed) = opts.seed {
        cvars.d_seed = seed;
    }
    if cvars.d_seed == 0 {
        let now = std::time::SystemTime::now();
        let secs = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
        cvars.d_seed = secs;
    }

    // LATER Load texture list and map in parallel with other assets
    let tex_list_bytes = load_file("assets/texture_list.txt").await.unwrap();
    draw_text("Loading...", 400.0, 400.0, 32.0, PURPLE);
    let tex_list_text = str::from_utf8(&tex_list_bytes).unwrap();
    let surfaces = map::load_tex_list(tex_list_text);
    let map_bytes = load_file(&opts.map).await.unwrap();
    draw_text("Loading...", 400.0, 400.0, 32.0, PURPLE);
    let map_text = str::from_utf8(&map_bytes).unwrap();
    let map = map::load_map(&map_text, surfaces);

    let time = Box::new(MacroquadTime);
    let mut server = Server::new(&cvars, time, map);

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

        if let Some(player2_handle) = player2_handle {
            server.input(player1_handle, mq::get_input1());
            server.input(player2_handle, mq::get_input2());
        } else {
            let input = mq::get_input1().merged(mq::get_input2());
            server.input(player1_handle, input);
        }

        server.update(&cvars, real_time);

        client.render(&server, &cvars);

        let before = get_time();
        next_frame().await;
        let after = get_time();
        client
            .rest_durations
            .add(cvars.d_timing_samples, after - before);
    }
}

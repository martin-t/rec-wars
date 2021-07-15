//! Entry point for the macroquad clients (both native and WASM).
//! The raw canvas + WASM client lives in lib.rs.

mod mq;

use std::str;

use ::rand::{prelude::SmallRng, SeedableRng};
use macroquad::prelude::*;
use structopt::StructOpt;

use rec_wars::{cvars::Cvars, map, server::Server, timing::MacroquadTime};

use crate::mq::MacroquadClient;

#[derive(StructOpt, Debug)]
// #[structopt(name = "basic")]
struct Opts {
    #[structopt(long)]
    splitscreen: bool,

    #[structopt(long, default_value = "maps/Atrium.map")]
    map: String,

    #[structopt(long, default_value = "7")] // TODO time?
    seed: u64,
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
    // TODO add CI for all build modes
    // TODO add all OSes to CI

    let opts = Opts::from_args();

    let mut cvars = Cvars::new_rec_wars();
    cvars.d_seed = opts.seed;
    let rng = SmallRng::seed_from_u64(cvars.d_seed);

    let tex_list_bytes = load_file("assets/texture_list.txt").await.unwrap();
    let tex_list_text = str::from_utf8(&tex_list_bytes).unwrap();
    let surfaces = map::load_tex_list(tex_list_text);
    let map_bytes = load_file(&opts.map).await.unwrap();
    let map_text = str::from_utf8(&map_bytes).unwrap();
    let map = map::load_map(&map_text, surfaces);

    let time = Box::new(MacroquadTime);
    let mut server = Server::new(&cvars, time, map, rng);

    let player1_handle = server.connect(&cvars, "Player 1");
    let player2_handle = if opts.splitscreen {
        Some(server.connect(&cvars, "Player 2"))
    } else {
        None
    };
    let mut client = MacroquadClient::new(&cvars, player1_handle, player2_handle).await;

    loop {
        let real_time = get_time();

        server.input(player1_handle, mq::get_input());

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

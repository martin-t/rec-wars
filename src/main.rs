//! Entry point for the macroquad clients (both native and WASM).
//! The raw canvas + WASM client lives in lib.rs.

mod mq;

use std::{env, str};

use ::rand::{prelude::SmallRng, SeedableRng};
use macroquad::prelude::*;

use rec_wars::{
    cvars::Cvars, entities::Player, game_state::GameState, map, server::Server,
    timing::MacroquadTime,
};

use crate::mq::MacroquadClient;

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
    // TODO move more init stuff from here to Server

    let mut args = env::args();
    args.next(); // Skip path of the executable
    let maybe_map = args.next();
    let maybe_seed = args.next();

    let cvars = Cvars::new_rec_wars();
    let rng = if let Some(seed) = maybe_seed {
        SmallRng::seed_from_u64(seed.parse().unwrap())
    } else if cvars.d_seed == 0 {
        SmallRng::seed_from_u64(7) // TODO time?
    } else {
        SmallRng::seed_from_u64(cvars.d_seed)
    };

    let tex_list_bytes = load_file("assets/texture_list.txt").await.unwrap();
    let tex_list_text = str::from_utf8(&tex_list_bytes).unwrap();
    let surfaces = map::load_tex_list(tex_list_text);
    let map_path = maybe_map.unwrap_or_else(|| "maps/Atrium.map".to_owned());
    let map_bytes = load_file(&map_path).await.unwrap();
    let map_text = str::from_utf8(&map_bytes).unwrap();
    let map = map::load_map(&map_text, surfaces);

    let mut gs = GameState::new(rng);
    let name = "Player 1".to_owned();
    let player = Player::new(name);
    let player1_handle = gs.players.insert(player);

    let time = Box::new(MacroquadTime);
    let mut client = MacroquadClient::new(&cvars, player1_handle).await;
    let mut server = Server::new(&cvars, time, map, gs);

    loop {
        let real_time = get_time();

        server.input(client.player_handle, mq::get_input());

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

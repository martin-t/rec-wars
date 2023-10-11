// Additional warnings that are allow by default (`rustc -W help`)
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unused)]
#![warn(clippy::all)]
#![allow(clippy::comparison_chain)] // Ifs are often cleaner with fewer indents
#![allow(clippy::iter_skip_next)] // Skip makes intent clearer sometimes
#![allow(clippy::needless_range_loop)] // False positives
#![allow(clippy::too_many_arguments)] // I decide what's too many

#[macro_use]
pub mod debug; // keep first so the macros are available everywhere

pub mod assets;
pub mod client;
pub mod common;
pub mod context;
pub mod cvars;
pub mod entities;
pub mod game_state;
pub mod input;
pub mod map;
pub mod net;
pub mod net_messages;
pub mod prelude;
pub mod rendering;
pub mod server;
pub mod sys_ai;
pub mod systems;
pub mod timing;
pub mod utils;
pub mod weapons;

use std::{env, error::Error, panic, process::Command};

use macroquad::prelude::*;

use crate::{net::Connection, prelude::*};

const BOT_NAMES: [&str; 20] = [
    "Dr. Dead",
    "Sir Hurt",
    "Mr. Pain",
    "PhD. Torture",
    "Mrs. Chestwound",
    "Ms. Dismember",
    "Don Lobotomy",
    "Lt. Dead",
    "Sgt. Dead",
    "Private Dead",
    "Colonel Dead",
    "Captain Dead",
    "Major Dead",
    "Commander Dead",
    "Díotóir",
    "Fireman",
    "Goldfinger",
    "Silverfinger",
    "Bronzefinger",
    "President Dead",
];

#[derive(Debug)]
enum Endpoint {
    /// Run a local game (client and server in one process)
    Local,
    /// Run only the game client
    Client,
    /// Run only the game server
    Server,
}

fn window_conf() -> Conf {
    // This is a hack because macroquad doesn't really allow modifying the window later.
    // LATER Find out what #[macroquad] expands to, do it manually
    //  but only if on client so server doesn't depend on macroquad.
    let arg = env::args().skip(1).next();
    let title = match arg.as_deref() {
        Some("server") => "RecWars Server",
        Some("client") => "RecWars Client",
        Some("local") => "RecWars Local",
        _ => "RecWars Launcher",
    };
    let (width, height) = match arg.as_deref() {
        Some("client" | "local") => (1600, 900),
        _ => (400, 200),
    };
    Conf {
        window_title: title.to_owned(),
        // In older macroquad, setting width and height to the size of the screen or larger
        // created a maximized window. This no longer works and the bottom OS panel covers
        // the window so this is an ugly compromise that should be good enough for most people.
        window_width: width,
        window_height: height,
        // LATER Allow resizing - AFAICT there was an issue with infinite memory growth when resizing the render target.
        // Can't use `fullscreen: true` because of https://github.com/not-fl3/macroquad/issues/237.
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> Result<(), Box<dyn Error>> {
    // We are not using a derive-based library (anymore)
    // because they add a couple hundred ms to incremental debug builds.
    //
    // If hand parsing gets too complex, might wanna consider one of the libs here:
    // https://github.com/rosetta-rs/argparse-rosetta-rs
    let mut args = env::args().skip(1).peekable(); // Skip path to self
    #[allow(unused)] // WASM hack below
    let mut endpoint = match args.peek().map(String::as_str) {
        Some("launcher") => {
            args.next();
            None
        }
        Some("local") => {
            args.next();
            Some(Endpoint::Local)
        }
        Some("client") => {
            args.next();
            Some(Endpoint::Client)
        }
        Some("server") => {
            args.next();
            Some(Endpoint::Server)
        }
        #[rustfmt::skip]
        Some("--help") => {
            println!("Usage: rec-wars [launcher|local|client|server] [cvar1 value1 cvar2 value2 ...]");
            println!();
            println!("Commands (optional):");
            println!("    launcher   Run a local game with separate client and server processes (default)");
            println!("    local      Run a local game with client and server in one process (experimental)");
            println!("    client     Run only the game client");
            println!("    server     Run only the dedicated game server");
            println!();
            println!("Cvars (optional):");
            println!("    You can specify cvars in key value pairs separated by space.");
            println!("    Example: rec-wars g_armor 150 hud_names false");
            println!();
            println!("    Cvars can be changed at runtime using the console but some of them");
            println!("    are only read at startup so the value needs to be specified");
            println!("    on the command line to take effect");
            println!();
            return Ok(());
        }
        Some("--version") => {
            // LATER Would be nice to print git hash and dirty status here.
            // Find a way to do that without increasing compile times or only do that in release builds.
            // Note that it's especially annoying when dirty status changes and forces a rebuild.
            // Maybe also include time of build.
            // https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
            println!("RecWars {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some(arg) if arg.starts_with('-') => {
            panic!("Unknown option: {}", arg);
        }
        _ => None,
    };
    // Anything else, we assume it's a cvar.
    // Some games require cvars/commands to be prefixed by `+` which allows more specific error messages
    // because they know it's meant to be a cvar/command and not a malformed command line option.
    // We might wanna require that too but this is slightly less typing for now.
    let cvar_args = args.collect();

    // Force local mode in WASM for now.
    #[cfg(target_arch = "wasm32")]
    {
        endpoint = Some(Endpoint::Local);
    }

    match endpoint {
        // LATER None should launch client and offer choice in menu
        None => {
            init_global_state("launcher");
            client_server_main(cvar_args).await;
        }
        Some(Endpoint::Local) => {
            init_global_state("lo");
            let cvars = args_to_cvars(&cvar_args)?;
            client_main(cvars, true).await;
        }
        Some(Endpoint::Client) => {
            init_global_state("cl");
            let cvars = args_to_cvars(&cvar_args)?;
            client_main(cvars, false).await;
        }
        Some(Endpoint::Server) => {
            init_global_state("sv");
            let cvars = args_to_cvars(&cvar_args)?;
            server_main(cvars).await;
        }
    }

    Ok(())
}

fn init_global_state(endpoint_name: &'static str) {
    debug::set_endpoint(endpoint_name);

    // Log which endpoint panicked.
    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        dbg_logf!("panicking"); // No need to print panic_info here, it'll be printed later anyway.
        prev_hook(panic_info);
    }));
}

fn args_to_cvars(cvar_args: &[String]) -> Result<Cvars, String> {
    let mut cvars = Cvars::default();

    let mut cvars_iter = cvar_args.iter();
    while let Some(cvar_name) = cvars_iter.next() {
        // Cvar names can optionally be prefixed by '+'.
        let mut cvar_name = cvar_name.as_str();
        if cvar_name.starts_with('+') {
            cvar_name = &cvar_name[1..];
        }

        let str_value = cvars_iter.next().ok_or_else(|| {
            format!("missing value for cvar `{cvar_name}` or incorrect command line option")
        })?;
        let res = cvars.set_str(cvar_name, str_value);
        match res.as_ref() {
            Ok(_) => {
                // Intentionally getting the new value from cvars, not just printing the input
                // so the user can check it was parsed correctly.
                dbg_logf!("{} = {}", cvar_name, cvars.get_string(cvar_name).unwrap());
            }
            Err(e) => {
                let msg = format!("failed to set cvar {cvar_name} to value {str_value}: {e}");
                if cvars.d_exit_on_unknown_cvar {
                    return Err(msg);
                } else {
                    dbg_logf!("WARNING {msg}");
                }
            }
        }
    }

    // Hack for web until there's a menu
    #[cfg(feature = "web_splitscreen")]
    {
        cvars.cl_splitscreen = true;
    }

    Ok(cvars)
}

/// Run both client and server.
///
/// This is just a convenience for quicker testing.
/// It spawns 2 processes to make sure the other is killed if one crashes.
///
/// LATER It should do that explicitly, right now it only kills the server
/// because client quits without a server anyway.
async fn client_server_main(cvar_args: Vec<String>) {
    draw_text("launcher", 10.0, 20.0, 16.0, RED);
    next_frame().await;

    let path = env::args().next().unwrap();

    let mut server_cmd = Command::new(&path);
    let mut client_cmd = Command::new(&path);

    server_cmd.arg("server");
    client_cmd.arg("client");

    for arg in &cvar_args {
        server_cmd.arg(arg);
        client_cmd.arg(arg);
    }

    let mut server = server_cmd.spawn().unwrap();
    // Sleep so the client window appears later and gets focus.
    std::thread::sleep(std::time::Duration::from_millis(50));
    let mut client = client_cmd.spawn().unwrap();

    // We wanna close just the client and automatically close the server that way.
    client.wait().unwrap();
    dbg_logf!("Client exited, killing server");
    server.kill().unwrap();
}

/// LATER Do we want a shared game state or just running both
/// client and server in one thread? Update docs on Endpoint or wherever.
async fn client_main(mut cvars: Cvars, local_game: bool) {
    dbg_logd!(local_game); // LATER Actually use

    show_mouse(false);

    let assets = Assets::load_all().await;

    // TODO Maybe extract connect/init into a function?
    //  Depends on how UI will work. Ideally calls to next_frame would all be in one place.
    //  Use https://github.com/optozorax/egui-macroquad when it supports macroquad 0.4,
    //  waiting on https://github.com/not-fl3/egui-miniquad/pull/63.
    //  Option 1: state machine: menu, connecting, playing - one main loop for all.
    //  Option 2: separate "main" loops, after game loop ends, return back into menu loop.

    draw_text(
        &format!("Connecting to {}...", &cvars.cl_net_server_addr),
        200.0,
        200.0,
        32.0,
        RED,
    );
    next_frame().await;

    let mut conn: Box<dyn Connection<ServerMessage>> =
        Box::new(net::tcp_connect_blocking(&cvars, &cvars.cl_net_server_addr));

    // LATER(splitscreen) handle 2 networked players on 1 connection (need to tell server how many players to spawn)
    let connect = Connect {
        cl_version: env!("GIT_VERSION").to_owned(),
        name1: cvars.cl_name1.clone(),
        name2: None,
    };
    let msg = ClientMessage::Connect(connect);
    let net_msg = net::serialize(msg);
    let res = conn.send(&net_msg);
    res.unwrap(); // LATER

    draw_text("Waiting for initial data...", 200.0, 200.0, 32.0, RED);
    next_frame().await;

    let init = loop {
        let (msg, closed) = conn.receive_one();
        if closed {
            dbg_logf!("Server disconnected");
            return;
        }

        if let Some(msg) = msg {
            match msg {
                ServerMessage::Init(init) => break init,
                msg => dbg_logf!("WARNING: Unexpected message type: {:?}", msg),
            }
        }

        next_frame().await;
    };

    // Using destructuring here so we get an error if a field is added but not read.

    let Init {
        sv_version,
        map_path,
        frame_num,
        game_time,
        game_time_prev,
        dt,
        players,
        local_player1_index,
        local_player2_index,
        vehicles,
        projectiles,
    } = init;
    assert!(local_player2_index.is_none()); // LATER

    let cl_version = env!("GIT_VERSION");
    dbg_logf!("Server version: {}", sv_version);
    dbg_logf!("Client version: {}", cl_version);
    if sv_version == cl_version {
        dbg_logf!("Versions match");
    } else {
        dbg_logf!("WARNING: Client and server versions don't match");
    }

    let map = load_map(&assets, &map_path);
    let mut gs = GameState::new();
    gs.frame_num = frame_num;
    gs.game_time = game_time;
    gs.game_time_prev = game_time_prev;
    gs.dt = dt;

    let mut ctx = FrameCtx::new(&cvars, &map, &mut gs);
    for player in players {
        ctx.init_player(player);
    }
    for vehicle in vehicles {
        ctx.init_vehicle(vehicle);
    }
    for projectile in projectiles {
        ctx.init_projectile(projectile);
    }

    let player1_handle = gs.players.slot_to_index(local_player1_index).unwrap();
    let mut client = Client::new(&cvars, assets, map, gs, conn, player1_handle, None);

    loop {
        // Input is outside the game loop because
        // - It needs access to the console and ClientCtx doesn't have it.
        // - Macroquad only updates input once per frame anyway.
        client.cl_input(&cvars);

        client.update(&cvars, get_time());

        client.render(&cvars);

        client.console.update(&mut cvars);

        client.post_render(&cvars);

        let before = get_time();
        next_frame().await;
        let after = get_time();
        let samples_max = cvars.d_timing_samples;
        client.engine_durations.add(samples_max, after - before);
    }
}

async fn server_main(mut cvars: Cvars) {
    init_seed(&mut cvars);
    let assets = Assets::load_all().await;

    let map_path = select_map(&mut cvars, &assets).to_owned();
    let map = load_map(&assets, &map_path);
    let mut server = Server::new(&cvars, map);

    loop {
        server.update(&cvars, get_time());

        // Draw something ever frame so we know the server is not stuck.
        clear_background(BLACK);
        draw_text(
            &format!("engine time {:.04}", get_time()),
            10.0,
            20.0,
            16.0,
            RED,
        );

        next_frame().await;
    }
}

fn init_seed(cvars: &mut Cvars) {
    if cvars.d_seed == 0 {
        let time_seed = macroquad::miniquad::date::now();
        cvars.d_seed = time_seed.to_bits();
    }
    dbg_logf!("Seed: {}", cvars.d_seed);
}

fn select_map<'a>(cvars: &'a mut Cvars, assets: &'a Assets) -> &'a str {
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
    map_path
}

fn load_map(assets: &Assets, map_path: &str) -> Map {
    let map_text = assets.maps.get(map_path).unwrap();
    let surfaces = map::parse_texture_list(&assets.texture_list);
    map::parse_map(map_text, surfaces, map_path)
}

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

pub mod client;
pub mod cvars;
pub mod entities;
pub mod game_state;
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

use std::{env, error::Error, panic, process::Command};

use macroquad::prelude::*;

use crate::{client::MacroquadClient, prelude::*, server::Server};

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
            client_server_main(cvar_args);
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
            server_main(cvars);
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
fn client_server_main(cvar_args: Vec<String>) {
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
async fn client_main(mut cvars: Cvars, _local_server: bool) {
    show_mouse(false);

    if cvars.d_seed == 0 {
        let time_seed = macroquad::miniquad::date::now();
        cvars.d_seed = time_seed.to_bits();
    }
    dbg_logf!("Seed: {}", cvars.d_seed);

    // LATER This doesn't display in native, what about web?
    draw_text("Loading...", 400.0, 400.0, 32.0, RED);
    let assets = Assets::load_all().await;
    let map_path = assets.select_map(&mut cvars);
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

fn server_main(_cvars: Cvars) {}

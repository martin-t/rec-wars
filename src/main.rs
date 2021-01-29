// Master TODO list:
// MVP:
// [x] pick a math lib, don't overthink it
// [x] load maps
// [x] cvars
//      https://crates.io/crates/cvar
//          no tab completion in js console
//          https://crates.io/crates/amethyst-console - uses cvar and imgui
// [x] some way to have references between entities
//      hecs
//          - can't delete while iterating
//          - no clone / snapshots
//          + ffa example
//          + nested queries (takes &World even for write access)
//          + slightly faster in WASM, especially debug mode
//          - no resources, commands
//      legion
//          - can't delete while iterating
//          + should be able to make a clone of the world (clone_from)
//          - bad docs / no examples / 0.3 transition
//          - how to do nested queries without copying? split world? maybe systems make this easier?
//          + slightly slower in WASM, especially debug mode
//          + resources, commands?
//      generational arena
//          + statically typed, simple
// [x] explosions
//      [x] sizes
// [x] hp/health
//      [x] wrecks (vehicle turned into wreck immediately on hit, before explosion covers it)
//      [x] configurable
// [x] player separate from vehicle
// [x] icon
// [x] version info
// [ ] mostly working weapons
// [x] respawning
// [x] scores
// [ ] primitive bots
// [ ] splitscreen
// [ ] decent default binds (2x for splitscreen)
// [ ] fix perf - canvas 2D is slow, pick an engine or rendering crate
//      option 1: optimize canvas (but still need something else for the native client)
//          https://isaacsukin.com/news/2015/01/detailed-explanation-javascript-game-loops-and-timing
//          render background to off-screen canvas, draw part of it to the main canvas in one call (actually 2 for nonwalls + walls)
//      option 2: pick a rust engine / rendering crate
//      engines
//          [x] check https://arewegameyet.rs/ecosystem/engines/
//          [x] check https://github.com/dasifefe/rust-game-development-frameworks
//          godot
//          https://crates.io/crates/amethyst
//              WASM in progress: https://github.com/amethyst/amethyst/issues/2260
//          https://crates.io/crates/bevy
//              no WASM yet: https://github.com/bevyengine/bevy/issues/88
//          https://crates.io/crates/coffee
//              no WASM: https://github.com/hecrj/coffee/issues/135 (not even a reply)
//          https://crates.io/crates/ggez
//              WASM in progress: https://github.com/ggez/ggez/issues/71
//          https://crates.io/crates/macroquad
//              win/lin/mac + WASM + android
//              bad docs, some examples
//          https://crates.io/crates/piston
//              no WASM: https://github.com/PistonDevelopers/piston/issues/1131
//          https://crates.io/crates/quicksilver
//              win/lin/max + WASM
//              no audio in 0.4?
//                  https://github.com/ryanisaacg/quicksilver/issues/552
//                  https://github.com/RustyVolley/RustyVolleySrc in 0.3 has sound
//              8 example games in readme (mostly bad)
//      rendering only
//          https://crates.io/crates/luminance - mentions webgl/canvas
//          https://crates.io/crates/miniquad - should support WASM
//          wgpu?
//      profiling
//          [ ] make explosion sprite smaller
//          https://github.com/EmbarkStudios/puffin
//          https://github.com/bombomby/optick-rs
//          list of tools: Instruction counts @ https://blog.mozilla.org/nnethercote/2020/09/08/how-to-speed-up-the-rust-compiler-one-last-time/
//      note to self - renderdoc for graphics debugging
//      [ ] FAQ - stuttering/tearing due to compositor - Alt+Shift+f12 - somehow this doesn't work anymore
// 1.0:
// [x] CI - GH actions
// [ ] extract/screenshot/record assets from RecWar or find alternatives
//      [ ] images
//          [x] weapon icons
//          [x] weapons
//          [ ] vehicles + skins
//          [ ] cow, stolen effects
//          [x] wrecks
//      [ ] sounds
//          [ ] weapons, explosions, self destruct
//          [ ] cow
//          [ ] engine noise
//              [ ] how does it change with speed?
// [ ] render vehicles
//      [x] basic tank
//      [ ] skins, colors
//              canvas imageData?
//      [x] vehicle types
// [ ] indicators for off-screen vehicles
// [ ] movement
//      [x] tank
//      [x] hovercraft
//      [ ] hummer + reverse steering
//          https://engineeringdotnet.blogspot.com/2010/04/simple-2d-car-physics-in-games.html
//          https://www.asawicki.info/Mirror/Car%20Physics%20for%20Games/Car%20Physics%20for%20Games.html
//      [ ] measure exact values to replicate RecWar
// [ ] collision detection with proper traces
//      [x] projectiles X walls
//      [x] projectiles X vehicles
//      [ ] vehicles X walls
//      [ ] vehicles X vehicles
// [ ] physics
//      [ ] surfaces
//      [ ] pushing wrecks
//      [ ] pushing moving vehicles
// [ ] weapons
//      [ ] MG
//          [x] 50 per reload
//          [x] about 2.5 s to empty magazine -> refire 50 ms
//          [ ] shoot faster on hovercraft, more ammo too?
//          [ ] shooting to the side while moving looks awful
//      [ ] rail
//          [x] push
//          [ ] exact push values - just add to velocity or divide by mass?
//          [x] shoot through vehicles (don't stop after first hit)
//      [ ] CB
//          [x] 40 per shot
//          [x] next shot starts a few frames after last explosion disappears
//          [x] explosions happen on walls, just partially obscured
//          [ ] reflect off map edge
//          [ ] hummer - originate from entire width of turret
//          orig RW:
//              size 1, shadow offset 1
//              40 frames to first explosion, 58 to last
//              distance - 80-300 while moving slowly backwards
//              damage hard to measure but looks similar to rockets
//      [ ] rockets
//          [ ] acceleration (judging by changing distance between them)
//          [x] 6 per reload
//          [x] about 1 s between first and last rocket -> refire 200 ms
//          [ ] hummer more + refire + entire width
//          [ ] orig RW: only direct hit does dmg, not explosion
//      [ ] homing missile
//      [ ] guided missile
//      [ ] BFG
//          [ ] shot delay + random dir when dead
//          [ ] beam range
//          [x] explosion animation goes backwards
//          My explosion spritesheet is slightly different from original RW, i experimented with GIMP and this is the closest i got.
//              Would be nice to figure out what exactly it did.
//          In orig RW, shooting at a wall could kill enemy on the other side but only if low hp - probably clipped into wall and beam killed him.
//          [ ] flashing color
//      [ ] missiles+BFG in orig RW: explosion range counted towards origin, not nearest, touches chassis if rotated towards explosion (origin is in turret swivel point)
//      [x] proper shot origins
//      [x] proper reload times
//      [ ] rockets?, missiles and bfg seem to have terminal velocity - force+friction?
//      [x] avoid hitting self (orig RW had hummer chassis hardpoint more forward)
//          [ ] allow hitting self if missile comes back after clearing the hitbox
//      [ ] entity culling? faster to render?
//      [ ] check collision detection works if gun is inside another vehicle
// [ ] mines
//      [ ] not within radius of base and/or cow
// [x] turrets
//      [x] 8 angles
//      [ ] better control scheme - changing direction cancels the queue, starts counting from current position
//          original RW has a bug: quick left,left,right would result in turning the longer way around
// [ ] shadows
//      [ ] HUD - all text for readability (score, scoreboard, names), weap, optionally hp/ammo
//      [ ] vehicles (is turret "higher"?), cow?, projectiles?
//      [x] CB
//      [ ] solve perf issues - firefox (78.0.2) *really* hates shadows (maybe only on text)
//              - try prerendering text into offscreen canvas
// [ ] self destruct
//      [x] bigger explosion
//      [ ] countdown
//      [ ] sounds
//      [ ] works through walls
//      [ ] seems damage depends on distance:
//          ~150 at center
//          range more than 64+64+32 with 40 health - maybe 0 dmg at ~175
// [ ] UI
//      requirements
//          minimum: select mode, map, bots, start game
//          server list? quick join? start server?
//          tank customization (only pattern, colors, name - vehicle type should be changeable in-game)
//          binds (show keyboard image?)
//              comfortable for 2 players, discoverable
//              check all the downloaded RW configs, RW2 and RW3 (copy config / firejail)
//      maybe https://www.sitepoint.com/css-layouts-floats-flexbox-grid/
// [ ] fullscreen
// [ ] FFA
// [ ] TW
// [ ] CTC
//      [ ] cow movement
//      crab instead? easter-egg only (temporarily? - e.g. after making a circlearound)?
// [ ] bot AI
//      [ ] pathfinding - custom / from soko solver / crate?
//          not just 8 directions but any angle
//          look at what veloren does
//          https://old.reddit.com/r/rust_gamedev/comments/hr7m3j/which_lib_do_you_use_for_pathfinding_in_you_games/
//          https://github.com/BezPowell/blitz-path
//          https://github.com/Winsalot/AutumnRTS/
//      [ ] roles / commands
// [ ] hosting
//      GH pages? - needs public repo first
//      domain? SSL?
//      master server? cloudflare, heroku, google app engine?
//          needs to be stateful
//          https://github.com/Ralith/masterserve
//      if dedicated servers, need at least 2 - EU and US
//          hardcoded list of servers as backup for master
// [ ] network multiplayer
//      WASM doesn't allow UDP sockets
//          https://gafferongames.com/post/why_cant_i_send_udp_packets_from_a_browser/
//          https://www.html5rocks.com/en/tutorials/webrtc/datachannels/
//          TCP
//              reduce extra latency: https://lobste.rs/s/5qlb7z/40_milliseconds_latency_just_would_not_go (article + link to HN)
//              crazy idea: multiple TCP streams, rotate among them - not all will be stalled by PL
//              https://old.reddit.com/r/linux/comments/k2opr7/veloren_08_an_opensource_multiplayer_rpg_is_being/gdz8avd/
//                  "never use TCP. ISPs will rudely inject RSTs into long-running streams, and you can't realistically ask people to install a firewall rule to drop them.
//                  "Likewise, always encrypt everything. QUIC made life easier but there are numerous alternates. Multiple streams are probably overrated."
//          webRTC
//      https://arewegameyet.rs/ecosystem/networking/ or custom?
//          https://crates.io/crates/quinn - what is QUIC?
//          https://crates.io/crates/laminar - mentions only UDP, not WASM
//          https://crates.io/crates/naia-server
//      [ ] prediction / reconciliation
//          https://github.com/pond3r/ggpo/tree/master/doc
//      [ ] server framerate when minimized - https://developer.mozilla.org/en-US/docs/Web/API/Window/setImmediate#Notes
//          postMessage / MessageChannel / https://github.com/YuzuJS/setImmediate polyfill
//      [ ] what happens to Entity handles after a player disconnects
// [ ] chat
// [ ] voting
//      [ ] mode
//      [ ] map
//      [ ] cvars
// [ ] focus canvas on (re)load (e.g. after touching browser console and reloading)
// nice to have:
// [ ] logo (RecWars spelled out by in-game entities?)
// [ ] GM - presing fire again switches back to tank
// [ ] replays (also easier debugging)
//      [ ] record seed+input (WASM should be reproducible when avoiding NaNs)
//      [ ] SmallRng depends on platform and rand version: https://docs.rs/rand/*/rand/rngs/struct.SmallRng.html
// [ ] allow MG to shoot down missiles
// [ ] lateral friction
// [ ] announcer
//      [ ] killstreaks? orig RW only if fast enough
//      [ ] other events - CTC steal/capture/return - need sounds from other games
// [ ] map editor - sharing maps, voting, recommended mode / number of bots
//      [ ] bots say hi/gg/sry/n1
// [ ] log of past games (to show activity even if nobody currently online)
// [ ] figure out what webpack is and how to create a static site
//      probably best template: https://github.com/rustwasm/rust-webpack-template
//      alternative: https://github.com/thedodd/trunk
//      note to self for dealing with npm on debian:
//          https://stackoverflow.com/questions/16151018/npm-throws-error-without-sudo/24404451#24404451
//          put in the *beginning* of PATH or it uses old debian npm which breaks everything
// [ ] native binary
// [ ] make parsing return errors instead of crashing
// [x] pause, variable speed
// [ ] frame debug mode - only render gamelogic frames, no interpolation
// [ ] shield pickups
// [ ] horn
//      [ ] sound, bind
//      [ ] make AI move out of the way
// [ ] cvar to set origin - tank in original RW turned around turret swivel point
// [ ] better cvar system - requirements:
//      [ ] in game console
//          [ ] autocompletion
//      [ ] config files - separate configs for RecWar and RecWars (one overriding just changed cvars from the other vs 2 whole configs?)
//      [ ] allow sharing/including other config files
//      [ ] generate struct from config?
//          [ ] accessors generic over weap/vehicle: g_[weapon].damage
// [ ] easter eggs
//      [ ] server say for 0 deaths
//      [ ] bugfeatures from original RW:
//          [ ] BFG can shoot through walls touching on the corner (well, somewhat)
//              detect when doing multiple times and print a message
//                  private?
//                  server say? (lol no collision detection?)
//          [ ] BFG does a tiny bit of dmg when hitting a wall with a tank on the other side
//              probably BFG briefly enters wall before collision is detected and does beam dmg
//          [ ] tank could shoot through a wall tile by putting the turret inside
// [ ] cleanup unused stuff from assets
// [ ] code cleanup
//      [ ] replace `as` with safer conversions (https://docs.rs/num_enum/0.5.1/num_enum/ instead of enumn?)
//              also vek's `as_`?
//      [ ] `unwrap` - they are all temporary from the prototyping phase
//              - some are sanity checks when removing from thunderdome - soft_unwrap instead?
//              - review all of them and eliminate or replace with `except` (to mark as reviewed)
//      [ ] find a way to increase rustfmt line length - the arbitrary line breaks are dumb
//              - stuff like setting canvas shadow offset should be one line
//              - stuff like long chains of iterator ops should be split
// [ ] all the LATERs - they mean something can be done better but marking it as a todo would be just noise when grepping

use std::fs; // FIXME mq::fs?

// FIXME breaks WASM+mq
use ::rand::{prelude::SmallRng, SeedableRng};
use macroquad::prelude::*;
use thunderdome::Index;

use rec_wars::{
    cvars::Cvars,
    entities::Player,
    game_state::GameState,
    map::{self, Kind, Vec2f, TILE_SIZE},
    server::Server,
    timing::{Fps, MacroquadTime},
};
use vek::Clamp;

#[derive(Debug)]
struct MacroquadClient {
    render_fps: Fps,
    player_handle: Index,
}

#[macroquad::main("RecWars")]
async fn main() {
    // TODO add CI for all build modes
    // TODO add all OSes to CI
    // TODO move more init stuff from here to Server

    let cvars = Cvars::new_rec_wars();
    let rng = if cvars.d_seed == 0 {
        // This requires the `wasm-bindgen` feature on `rand` or it crashes at runtime.
        SmallRng::from_entropy()
    } else {
        SmallRng::seed_from_u64(cvars.d_seed)
    };

    let mut imgs_tiles = Vec::new();
    for path in &[
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
    ] {
        imgs_tiles.push(load_texture(path).await);
    }

    let tex_list_text = fs::read_to_string("assets/texture_list.txt").unwrap();
    let surfaces = map::load_tex_list(&tex_list_text);
    let map_text = fs::read_to_string("maps/Atrium.map").unwrap();
    let map = map::load_map(&map_text, surfaces);

    let mut gs = GameState::new(rng);
    let name = "Player 1".to_owned();
    let player = Player::new(name);
    let player1_handle = gs.players.insert(player);

    let time = Box::new(MacroquadTime);
    let client = MacroquadClient {
        render_fps: Fps::new(),
        player_handle: player1_handle,
    };
    let mut server = Server::new(&cvars, time, map, gs);

    // client: RawCanvasClient {
    //     canvas,
    //     context,
    //     imgs_tiles,
    //     imgs_vehicles,
    //     imgs_wrecks,
    //     imgs_weapon_icons,
    //     img_rocket,
    //     img_hm,
    //     img_gm,
    //     img_explosion,
    //     img_explosion_cyan,
    //     render_fps: Fps::new(),
    //     render_durations: Durations::new(),
    //     player_handle: player1_handle,
    // },

    let texture = load_texture("assets/tiles/base.bmp").await;

    loop {
        let start = get_time();

        server.update(&cvars, start);

        // TODO smoothing?

        let player = &server.gs.players[client.player_handle];
        let player_veh_pos = server.gs.vehicles[player.vehicle.unwrap()].pos;
        let player_entity_pos = if let Some(gm_handle) = player.guided_missile {
            server.gs.projectiles[gm_handle].pos
        } else {
            player_veh_pos
        };

        // Don't put the camera so close to the edge that it would render area outside the map.
        // TODO handle maps smaller than canvas (currently crashes on unreachable)
        let view_size = Vec2f::new(screen_width() as f64, screen_height() as f64);
        let camera_min = view_size / 2.0;
        let map_size = server.map.maxs();
        let camera_max = map_size - camera_min;
        let camera_pos = player_entity_pos.clamped(camera_min, camera_max);

        // Position of the camera's top left corner in world coords.
        // Subtract this from world coords to get screen coords.
        // Forgetting this is a recurring source of bugs.
        // I've considered making a special type for screen coords (e.g. struct Vec2screen(Vec2f);)
        // so you couldn't accidentally pass world coords to drawing fns but it turned out to be more work than expected:
        // - The newtype had to manually impl all the needed operations of the underlying Vec2 type because ops don't autoderef.
        // - What would be the result of ops that take one world coord and one screen coord? Lots of cases to think about.
        // - Which type are sizes? E.g. `center = corner + size/2` makes sense in both screen and world coords.
        let top_left = camera_pos - camera_min;

        let top_left_tp = server.map.tile_pos(top_left);
        let top_left_index = top_left_tp.index;
        let bg_offset = if cvars.r_align_to_pixels_background {
            top_left_tp.offset.floor()
        } else {
            top_left_tp.offset
        };

        // Draw non-walls
        let mut r = top_left_index.y;
        let mut y = -bg_offset.y;
        while y < view_size.y {
            let mut c = top_left_index.x;
            let mut x = -bg_offset.x;
            while x < view_size.x {
                let tile = server.map.col_row(c, r);

                if server.map.surface_of(tile).kind != Kind::Wall {
                    let img = imgs_tiles[tile.surface_index];
                    draw_tile(img, x, y, tile.angle);
                }

                c += 1;
                x += TILE_SIZE;
            }
            r += 1;
            y += TILE_SIZE;
        }

        // Draw walls
        // They are above explosions and turrets, just like in RecWar.
        let mut r = top_left_index.y;
        let mut y = -bg_offset.y;
        while y < view_size.y {
            let mut c = top_left_index.x;
            let mut x = -bg_offset.x;
            while x < view_size.x {
                let tile = server.map.col_row(c, r);

                if server.map.surface_of(tile).kind == Kind::Wall {
                    let img = imgs_tiles[tile.surface_index];
                    draw_tile(img, x, y, tile.angle);
                }

                c += 1;
                x += TILE_SIZE;
            }
            r += 1;
            y += TILE_SIZE;
        }

        let end = get_time();
        draw_text(&get_fps().to_string(), 400.0, 300.0, 20.0, WHITE);
        draw_text(&get_frame_time().to_string(), 400.0, 330.0, 20.0, WHITE);
        draw_text(&get_time().to_string(), 400.0, 360.0, 20.0, WHITE);
        draw_text(&(end - start).to_string(), 400.0, 390.0, 20.0, WHITE);
        next_frame().await
    }
}

fn draw_tile(img: Texture2D, x: f64, y: f64, angle: f64) {
    draw_texture_ex(
        img,
        x as f32,
        y as f32,
        WHITE,
        DrawTextureParams {
            rotation: angle as f32,
            pivot: Some(Vec2::new(
                x as f32 + TILE_SIZE as f32 / 2.0,
                y as f32 + TILE_SIZE as f32 / 2.0,
            )),
            ..Default::default()
        },
    )
}

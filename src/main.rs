// Master TODO list:
// MVP:
// [x] pick a math lib, don't overthink it
// [x] load maps
// [x] cvars
//      https://crates.io/crates/cvar
//          no tab completion in js console
//          https://crates.io/crates/amethyst-console - uses cvar and imgui
// [x] some way to have references between entities
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
// [x] pick an engine
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
// profiling
//      [ ] make explosion sprite smaller
//      https://github.com/EmbarkStudios/puffin
//      https://github.com/bombomby/optick-rs (Windows only)
//      list of tools: Instruction counts @ https://blog.mozilla.org/nnethercote/2020/09/08/how-to-speed-up-the-rust-compiler-one-last-time/
//      renderdoc for graphics debugging
//      cargo flamegraph (internally uses perf on linux)
//      firestorm
// [ ] FAQ - stuttering/tearing due to compositor - Alt+Shift+f12 - somehow this doesn't work anymore
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
// [ ] all the LATERs
//      - They mean something can be done better but marking it as a todo would be just noise when grepping.
//        They're things I'd do if I had infinite time and wanted to make RecWars perfect.

use std::{cmp::Reverse, env, str};

use ::rand::{prelude::SmallRng, SeedableRng};
use macroquad::prelude::*;
use thunderdome::Index;

use rec_wars::{
    cvars::Cvars,
    debugging::{DEBUG_CROSSES, DEBUG_LINES, DEBUG_TEXTS, DEBUG_TEXTS_WORLD},
    entities::{Ammo, Player, Weapon},
    game_state::{Explosion, GameState, Input},
    map::{self, F64Ext, Kind, Vec2f, VecExt, TILE_SIZE},
    server::Server,
    timing::{Durations, Fps, MacroquadTime},
};
use vek::Clamp;

#[derive(Debug)]
struct MacroquadClient {
    render_fps: Fps,
    render_cmds_durations: Durations,
    rest_durations: Durations,
    player_handle: Index,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "RecWars".to_owned(),
        // Setting width and height to the size of the screen or larger
        // creates a maximized window. Tested on Kubuntu 20.10.
        // Not using larger values (or i32::MAX) in case other platforms behave differently.
        window_width: 1920,
        window_height: 1080,
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
        // TODO load in parallel
        imgs_tiles.push(load_texture(path).await);
    }

    let mut imgs_vehicles = Vec::new();
    for path in &[
        "assets/vehicles/tank_chassis_flames.png",
        "assets/vehicles/tank_turret_flames.png",
        "assets/vehicles/hovercraft_chassis_flames.png",
        "assets/vehicles/hovercraft_turret_flames.png",
        "assets/vehicles/hummer_chassis_flames.png",
        "assets/vehicles/hummer_turret_flames.png",
    ] {
        imgs_vehicles.push(load_texture(path).await);
    }

    let mut imgs_wrecks = Vec::new();
    for path in &[
        "assets/wrecks/tank.png",
        "assets/wrecks/hovercraft.png",
        "assets/wrecks/hummer.png",
    ] {
        imgs_wrecks.push(load_texture(path).await);
    }

    let mut imgs_weapon_icons = Vec::new();
    for path in &[
        "assets/weapon_icons/mg.png",
        "assets/weapon_icons/rail.png",
        "assets/weapon_icons/cb.png",
        "assets/weapon_icons/rockets.png",
        "assets/weapon_icons/hm.png",
        "assets/weapon_icons/gm.png",
        "assets/weapon_icons/bfg.png",
    ] {
        imgs_weapon_icons.push(load_texture(path).await);
    }

    // LATER smoothing optional and configurable per image
    // LATER either use or remove r_smoothing (if raw_canvas is removed)
    let img_explosion = load_texture("assets/explosion.png").await;
    set_texture_filter(img_explosion, FilterMode::Nearest);
    let img_explosion_cyan = load_texture("assets/explosion_cyan.png").await;
    set_texture_filter(img_explosion_cyan, FilterMode::Nearest);
    let img_rocket = load_texture("assets/weapons/rocket.png").await;
    let img_hm = load_texture("assets/weapons/homing_missile.png").await;
    let img_gm = load_texture("assets/weapons/guided_missile.png").await;

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
    let mut client = MacroquadClient {
        render_fps: Fps::new(),
        render_cmds_durations: Durations::new(),
        rest_durations: Durations::new(),
        player_handle: player1_handle,
    };
    let mut server = Server::new(&cvars, time, map, gs);

    loop {
        let start = get_time();

        fn was_input_pressed(key_codes: &[KeyCode]) -> bool {
            for &key_code in key_codes {
                // Check both to avoid skipping input if it's pressed and released within one frame.
                if is_key_pressed(key_code) || is_key_down(key_code) {
                    return true;
                }
            }
            false
        }

        let mut input = Input::new();
        if was_input_pressed(&[KeyCode::Left, KeyCode::A]) {
            input.left = true;
        }
        if was_input_pressed(&[KeyCode::Right, KeyCode::D]) {
            input.right = true;
        }
        if was_input_pressed(&[KeyCode::Up, KeyCode::W]) {
            input.up = true;
        }
        if was_input_pressed(&[KeyCode::Down, KeyCode::S]) {
            input.down = true;
        }
        if was_input_pressed(&[KeyCode::Q, KeyCode::N]) {
            input.turret_left = true;
        }
        if was_input_pressed(&[KeyCode::E, KeyCode::M]) {
            input.turret_right = true;
        }
        if was_input_pressed(&[KeyCode::V, KeyCode::Period]) {
            input.prev_weapon = true;
        }
        if was_input_pressed(&[
            KeyCode::LeftShift,
            KeyCode::RightShift,
            KeyCode::B,
            KeyCode::Comma,
        ]) {
            input.next_weapon = true;
        }
        if was_input_pressed(&[KeyCode::Space]) {
            input.fire = true;
        }
        if was_input_pressed(&[KeyCode::J, KeyCode::X]) {
            input.mine = true;
        }
        if was_input_pressed(&[KeyCode::L]) {
            input.self_destruct = true;
        }
        if was_input_pressed(&[KeyCode::H]) {
            input.horn = true;
        }
        if was_input_pressed(&[]) {
            input.chat = true;
        }
        if was_input_pressed(&[KeyCode::Pause, KeyCode::P]) {
            input.pause = true;
        }
        server.input(client.player_handle, input);

        server.update(&cvars, start);

        // LATER when raw_canvas is removed, clean up all the casts here

        client.render_fps.tick(cvars.d_fps_period, server.real_time);

        let player = &server.gs.players[client.player_handle];
        let player_veh_pos = server.gs.vehicles[player.vehicle.unwrap()].pos;
        let player_entity_pos = if let Some(gm_handle) = player.guided_missile {
            server.gs.projectiles[gm_handle].pos
        } else {
            player_veh_pos
        };

        // Don't put the camera so close to the edge that it would render area outside the map.
        // Also properly handle maps smaller than view size. Note they can be smaller along X, Y or both.
        // Example maps for testing: Joust (2), extra/OK Corral (2)
        let screen_size = Vec2f::new(screen_width() as f64, screen_height() as f64);
        let map_size = server.map.maxs();
        let view_size = Vec2f::new(screen_size.x.min(map_size.x), screen_size.y.min(map_size.y));
        let empty_space_size = screen_size - view_size;
        let view_pos = empty_space_size / 2.0;

        // Camera center in world coords.
        let camera_pos_min = view_size / 2.0;
        let camera_pos_max = map_size - camera_pos_min;
        let camera_pos = player_entity_pos.clamped(camera_pos_min, camera_pos_max);

        // Position of the camera's top left corner in world coords.
        let camera_top_left = camera_pos - camera_pos_min;
        // Add this to world coords to get screen coords.
        // Forgetting to do this is a recurring source of bugs.
        // I've considered making a special type for screen coords (e.g. struct Vec2screen(Vec2f);)
        // so you couldn't accidentally pass world coords to drawing fns but it turned out to be more work than expected:
        // - The newtype had to manually impl all the needed operations of the underlying Vec2 type because ops don't autoderef.
        // - What would be the result of ops that take one world coord and one screen coord? Lots of cases to think about.
        // - Which type are sizes? Another type? E.g. `center = corner + size/2` makes sense in both screen and world coords.
        let camera_offset = -camera_top_left + view_pos;

        let top_left_tp = server.map.tile_pos(camera_top_left);
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
                    render_tile(img, view_pos.x + x, view_pos.y + y, tile.angle);
                }

                c += 1;
                x += TILE_SIZE;
            }
            r += 1;
            y += TILE_SIZE;
        }

        // Helper to filter projectiles by weapon.
        let weapon_projectiles = |weapon| {
            server
                .gs
                .projectiles
                .iter()
                .filter(move |(_, proj)| proj.weapon == weapon)
        };

        let outside_view_top_left = view_pos - TILE_SIZE;
        let outside_view_bottom_right = view_pos + view_size + TILE_SIZE;
        // Is the object certainly outside camera view?
        // Only works on objects smaller that tile size, which is most.
        // Exceptions are lines and text.
        let cull = |scr_pos: Vec2f| {
            scr_pos.x < outside_view_top_left.x
                || scr_pos.y < outside_view_top_left.y
                || scr_pos.x > outside_view_bottom_right.x
                || scr_pos.y > outside_view_bottom_right.y
        };

        // Draw MGs
        for (_, mg) in weapon_projectiles(Weapon::Mg) {
            let scr_pos = mg.pos + camera_offset;
            if cull(scr_pos) {
                continue;
            }
            // we're drawing from the bullet's position backwards
            let scr_end = scr_pos - mg.vel.normalized() * cvars.g_machine_gun_trail_length;
            render_line(scr_pos, scr_end, 1.0, YELLOW);
        }

        // Draw railguns
        for beam in &server.gs.rail_beams {
            let scr_begin = beam.begin + camera_offset;
            let scr_end = beam.end + camera_offset;
            render_line(scr_begin, scr_end, 1.0, Color::new(0.0, 0.0, 1.0, 1.0));
        }

        // Draw rockets, homing and guided missiles
        for (_, proj) in weapon_projectiles(Weapon::Rockets) {
            let scr_pos = proj.pos + camera_offset;
            if cull(scr_pos) {
                continue;
            }
            render_img_center(img_rocket, scr_pos, proj.vel.to_angle());
        }
        for (_, proj) in weapon_projectiles(Weapon::Hm) {
            let scr_pos = proj.pos + camera_offset;
            if cull(scr_pos) {
                continue;
            }
            render_img_center(img_hm, scr_pos, proj.vel.to_angle());
        }
        for (_, proj) in weapon_projectiles(Weapon::Gm) {
            let scr_pos = proj.pos + camera_offset;
            if cull(scr_pos) {
                continue;
            }
            render_img_center(img_gm, scr_pos, proj.vel.to_angle());
        }

        // Draw BFGs
        // client.context.set_fill_style(&"lime".into());
        // client.context.set_stroke_style(&"lime".into());
        for (_, bfg) in weapon_projectiles(Weapon::Bfg) {
            let scr_pos = bfg.pos + camera_offset;
            if cull(scr_pos) {
                continue;
            }
            draw_circle(
                scr_pos.x as f32,
                scr_pos.y as f32,
                cvars.g_bfg_radius as f32,
                GREEN,
            );
        }
        for &(src, dest) in &server.gs.bfg_beams {
            let scr_src = src + camera_offset;
            let scr_dest = dest + camera_offset;
            render_line(scr_src, scr_dest, 1.0, GREEN);
        }

        // Draw chassis
        for (_, vehicle) in server.gs.vehicles.iter() {
            let scr_pos = vehicle.pos + camera_offset;
            if cull(scr_pos) {
                continue;
            }
            let img;
            if vehicle.destroyed() {
                img = imgs_wrecks[vehicle.veh_type as usize];
            } else {
                img = imgs_vehicles[vehicle.veh_type as usize * 2];
            }
            render_img_center(img, scr_pos, vehicle.angle);
            // LATER draw hitboxes
            // if cvars.d_draw && cvars.d_draw_hitboxes {
            //     client.context.set_stroke_style(&"yellow".into());
            //     client.context.begin_path();
            //     let corners = vehicle.hitbox.corners(scr_pos, vehicle.angle);
            //     move_to(client, corners[0]);
            //     line_to(client, corners[1]);
            //     line_to(client, corners[2]);
            //     line_to(client, corners[3]);
            //     client.context.close_path();
            //     client.context.stroke();
            // }
        }

        // TODO Draw cow

        // Draw turrets
        for (_, vehicle) in server.gs.vehicles.iter() {
            if vehicle.destroyed() {
                continue;
            }

            let scr_pos = vehicle.pos + camera_offset;
            if cull(scr_pos) {
                continue;
            }

            let img = imgs_vehicles[vehicle.veh_type as usize * 2 + 1];
            let offset_chassis =
                vehicle.angle.to_mat2f() * cvars.g_vehicle_turret_offset_chassis(vehicle.veh_type);
            let turret_scr_pos = scr_pos + offset_chassis;
            let offset_turret = cvars.g_vehicle_turret_offset_turret(vehicle.veh_type);
            render_img_offset(
                img,
                turret_scr_pos,
                vehicle.angle + vehicle.turret_angle_current,
                offset_turret,
            );
        }

        // Draw explosions
        let iter: Box<dyn Iterator<Item = &Explosion>> = if cvars.r_explosions_reverse {
            Box::new(server.gs.explosions.iter().rev())
        } else {
            Box::new(server.gs.explosions.iter())
        };
        for explosion in iter {
            let scr_pos = explosion.pos + camera_offset;
            if cull(scr_pos) {
                continue;
            }

            // It looks like the original animation is made for 30 fps.
            // Single stepping a recording of the original RecWars explosion in blender:
            // 13 sprites, 31 frames - examples:
            //      2,2,3,1,3,3,2,3,2,2,3,2,3
            //      2,2,2,3,1,3,2,2,3,2,2,3,4
            // Different each time probably because RecWar's and the recorder's framerate don't match exactly.
            //
            // This code produces similar results,
            // though it might display a single sprite for 4 frames slightly more often.
            let progress =
                (server.gs.game_time - explosion.start_time) / cvars.r_explosion_duration;
            // 13 sprites in the sheet, 100x100 pixels per sprite
            let frame = (progress * 13.0).floor();
            let (offset, img);
            if explosion.bfg {
                offset = (12.0 - frame) * 100.0;
                img = img_explosion_cyan;
            } else {
                offset = frame * 100.0;
                img = img_explosion;
            };
            draw_texture_ex(
                img,
                (scr_pos.x - 50.0 * explosion.scale) as f32,
                (scr_pos.y - 50.0 * explosion.scale) as f32,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(
                        100.0 * explosion.scale as f32,
                        100.0 * explosion.scale as f32,
                    )),
                    source: Some(Rect::new(offset as f32, 0.0, 100.0, 100.0)),
                    ..Default::default()
                },
            );
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
                    render_tile(img, view_pos.x + x, view_pos.y + y, tile.angle);
                }

                c += 1;
                x += TILE_SIZE;
            }
            r += 1;
            y += TILE_SIZE;
        }

        // Draw cluster bombs
        // TODO what about shadows (in general)?
        if cvars.r_draw_cluster_bombs {
            for (_, cb) in weapon_projectiles(Weapon::Cb) {
                let scr_pos = cb.pos + camera_offset;
                if cull(scr_pos) {
                    continue;
                }

                let corner = scr_pos - cvars.g_cluster_bomb_size / 2.0;
                // Tecnically, we should draw all shadows first, then all the projectiles,
                // but actually it barely matters and I think RecWar does it this way too.
                draw_rectangle(
                    (corner.x + cvars.g_cluster_bomb_shadow_x) as f32,
                    (corner.y + cvars.g_cluster_bomb_shadow_y) as f32,
                    cvars.g_cluster_bomb_size as f32,
                    cvars.g_cluster_bomb_size as f32,
                    Color::new(0.0, 0.0, 0.0, cvars.g_cluster_bomb_shadow_alpha as f32),
                );
                draw_rectangle(
                    corner.x as f32,
                    corner.y as f32,
                    cvars.g_cluster_bomb_size as f32,
                    cvars.g_cluster_bomb_size as f32,
                    Color::new(0.0, 1.0, 1.0, 1.0),
                );
            }
        }

        // Draw world-space HUD elements:

        // Names
        if cvars.hud_names {
            for (_, vehicle) in server.gs.vehicles.iter() {
                let scr_pos = vehicle.pos + camera_offset;
                if cull(scr_pos) {
                    // LATER, restrict name length
                    continue;
                }

                let name = &server.gs.players[vehicle.owner].name;
                let size = measure_text(name, None, cvars.hud_names_font_size as u16, 1.0);
                // LATER remove cvars.hud_names_shadow_x/y when raw_canvas is removed
                render_text_with_shadow(
                    &cvars,
                    name,
                    scr_pos.x as f32 - size.width / 2.0,
                    (scr_pos.y + cvars.hud_names_y) as f32,
                    cvars.hud_names_font_size,
                    Color::new(
                        cvars.hud_names_brightness as f32,
                        cvars.hud_names_brightness as f32,
                        cvars.hud_names_brightness as f32,
                        cvars.hud_names_alpha as f32,
                    ),
                    cvars.hud_names_shadow_mq_x,
                    cvars.hud_names_shadow_mq_y,
                    cvars.hud_names_shadow_alpha,
                );
            }
        }

        // Homing missile indicator
        // TODO dashed lines (maybe use image)
        let player_veh_scr_pos = player_veh_pos + camera_offset;
        draw_circle_lines(
            player_veh_scr_pos.x as f32,
            player_veh_scr_pos.y as f32,
            cvars.hud_missile_indicator_radius as f32,
            1.0,
            GREEN,
        );
        let dir = 0.0.to_vec2f(); // TODO
        let end = player_veh_scr_pos + dir * cvars.hud_missile_indicator_radius;
        render_line(player_veh_scr_pos, end, 1.0, GREEN);

        // Debug lines and crosses
        // TODO colors (also in other places below)
        DEBUG_LINES.with(|lines| {
            let mut lines = lines.borrow_mut();
            for line in lines.iter_mut() {
                if cvars.d_draw && cvars.d_draw_lines {
                    let scr_begin = line.begin + camera_offset;
                    let scr_end = line.end + camera_offset;
                    render_line(scr_begin, scr_end, 1.0, RED);
                    if cvars.d_draw_lines_ends_length > 0.0 {
                        let segment = line.end - line.begin;
                        let perpendicular = Vec2f::new(-segment.y, segment.x).normalized();
                        render_line(
                            scr_begin + -perpendicular * cvars.d_draw_lines_ends_length,
                            scr_begin + perpendicular * cvars.d_draw_lines_ends_length,
                            1.0,
                            RED,
                        );
                        render_line(
                            scr_end + -perpendicular * cvars.d_draw_lines_ends_length,
                            scr_end + perpendicular * cvars.d_draw_lines_ends_length,
                            1.0,
                            RED,
                        );
                    }
                }
                line.time -= server.gs.dt;
            }
        });
        DEBUG_CROSSES.with(|crosses| {
            let mut crosses = crosses.borrow_mut();
            for cross in crosses.iter_mut() {
                if cvars.d_draw && cvars.d_draw_crosses {
                    let scr_point = cross.point + camera_offset;
                    if cull(scr_point) {
                        continue;
                    }

                    let top_left = scr_point - Vec2f::new(-3.0, -3.0);
                    let bottom_right = scr_point - Vec2f::new(3.0, 3.0);
                    let top_right = scr_point - Vec2f::new(3.0, -3.0);
                    let bottom_left = scr_point - Vec2f::new(-3.0, 3.0);
                    render_line(top_left, bottom_right, 1.0, RED);
                    render_line(top_right, bottom_left, 1.0, RED);
                }
                cross.time -= server.gs.dt;
            }
        });

        // Draw screen-space HUD elements:

        let mut player_points: Vec<_> = server
            .gs
            .players
            .iter()
            .map(|(index, player)| (index, player.score.points(&cvars)))
            .collect();
        player_points.sort_by_key(|&(_, points)| Reverse(points));

        // Score
        let score_pos = hud_pos(view_size, cvars.hud_score_x, cvars.hud_score_y);
        let points = player.score.points(&cvars).to_string();
        render_text_with_shadow(
            &cvars,
            &points,
            score_pos.x,
            score_pos.y,
            cvars.hud_score_font_size,
            WHITE,
            cvars.hud_score_shadow_mq_x,
            cvars.hud_score_shadow_mq_y,
            1.0,
        );

        // Ranking
        // Original RW shows "current rank / total players (+/- points difference to leader or second)"
        // as a big but not bold number with a 1px shadow. E.g. "1/3 (+5)" or "2/3 (0)".
        // There's no special treatement for players with the same number of points.
        let ranking_pos = hud_pos(view_size, cvars.hud_ranking_x, cvars.hud_ranking_y);
        let current_index = player_points
            .iter()
            .position(|&(handle, _)| handle == client.player_handle)
            .unwrap();
        let points_diff = if current_index == 0 {
            if player_points.len() == 1 {
                // The player is alone.
                0
            } else {
                player_points[current_index].1 - player_points[1].1
            }
        } else {
            player_points[current_index].1 - player_points[0].1
        };
        let ranking = if points_diff > 0 {
            // Only show the + sign for positive numbers, not 0
            format!(
                "{}/{} (+{})",
                current_index + 1,
                player_points.len(),
                points_diff
            )
        } else {
            format!(
                "{}/{} ({})",
                current_index + 1,
                player_points.len(),
                points_diff
            )
        };
        render_text_with_shadow(
            &cvars,
            &ranking,
            ranking_pos.x,
            ranking_pos.y,
            cvars.hud_ranking_font_size,
            WHITE,
            cvars.hud_ranking_shadow_mq_x,
            cvars.hud_ranking_shadow_mq_y,
            1.0,
        );

        // Hit points (goes from green to red)
        // Might wanna use https://crates.io/crates/colorsys if I need more color operations.
        // Hit points to color (poor man's HSV):
        // 0.0 = red
        // 0.0..0.5 -> increase green channel
        // 0.5 = yellow
        // 0.5..1.0 -> decrease red channel
        // 1.0 = green
        let player_vehicle = &server.gs.vehicles[player.vehicle.unwrap()];
        let r = 1.0 - (player_vehicle.hp_fraction.clamped(0.5, 1.0) - 0.5) * 2.0;
        let g = player_vehicle.hp_fraction.clamped(0.0, 0.5) * 2.0;
        let rgb = Color::new(r as f32, g as f32, 0.0, 1.0);
        let hp_pos = hud_pos(view_size, cvars.hud_hp_x, cvars.hud_hp_y);
        draw_rectangle(
            hp_pos.x,
            hp_pos.y,
            (cvars.hud_hp_width * player_vehicle.hp_fraction) as f32,
            cvars.hud_hp_height as f32,
            rgb,
        );
        if cvars.d_draw_text {
            let hp_number =
                player_vehicle.hp_fraction * cvars.g_vehicle_hp(player_vehicle.veh_type);
            let hp_text = format!("{}", hp_number);
            render_text_with_shadow(
                &cvars,
                &hp_text,
                hp_pos.x - 25.0,
                hp_pos.y + cvars.hud_hp_height as f32,
                16.0,
                RED,
                1.0,
                1.0,
                0.5,
            );
        }

        // Ammo
        let ammo = player_vehicle.ammos[player.cur_weapon as usize];
        let ammo_fraction = match ammo {
            Ammo::Loaded(_ready_time, count) => {
                let max = cvars.g_weapon_reload_ammo(player.cur_weapon);
                count as f64 / max as f64
            }
            Ammo::Reloading(start, end) => {
                let max_diff = end - start;
                let cur_diff = server.gs.game_time - start;
                cur_diff / max_diff
            }
        };
        let ammo_pos = hud_pos(view_size, cvars.hud_ammo_x, cvars.hud_ammo_y);
        draw_rectangle(
            ammo_pos.x,
            ammo_pos.y,
            (cvars.hud_ammo_width * ammo_fraction) as f32,
            cvars.hud_ammo_height as f32,
            YELLOW,
        );
        if cvars.d_draw_text {
            let ammo_number = match ammo {
                Ammo::Loaded(_ready_time, count) => count,
                Ammo::Reloading(_start, _end) => 0,
            };
            render_text_with_shadow(
                &cvars,
                &ammo_number.to_string(),
                ammo_pos.x - 25.0,
                ammo_pos.y + cvars.hud_ammo_height as f32,
                16.0,
                RED,
                1.0,
                1.0,
                0.5,
            );
        }

        // Weapon icon
        // The original shadows were part of the image but this is good enough for now.
        let weap_img = imgs_weapon_icons[player.cur_weapon as usize];
        let weap_icon_pos = hud_pos(view_size, cvars.hud_weapon_icon_x, cvars.hud_weapon_icon_y)
            - Vec2::new(weap_img.width(), weap_img.height()) / 2.0;
        draw_texture(
            weap_img,
            weap_icon_pos.x + cvars.hud_weapon_icon_shadow_mq_x,
            weap_icon_pos.y + cvars.hud_weapon_icon_shadow_mq_y,
            Color::new(0.0, 0.0, 0.0, cvars.hud_weapon_icon_shadow_alpha as f32),
        );
        draw_texture(weap_img, weap_icon_pos.x, weap_icon_pos.y, WHITE);

        // Scoreboard
        if player_vehicle.destroyed() {
            let width = cvars.hud_scoreboard_width_name
                + cvars.hud_scoreboard_width_kills
                + cvars.hud_scoreboard_width_deaths
                + cvars.hud_scoreboard_width_points;
            let height =
                (server.gs.players.len() + 1) as f32 * cvars.hud_scoreboard_line_height as f32;
            let x_start = (view_size.x as f32 - width) / 2.0;
            let mut x = x_start.floor();
            let mut y = ((view_size.y as f32 - height) / 2.0).floor();

            let fs = cvars.hud_scoreboard_font_size;
            let sx = cvars.hud_scoreboard_shadow_mq_x;
            let sy = cvars.hud_scoreboard_shadow_mq_y;

            // LATER bold header
            render_text_with_shadow(&cvars, "Name", x, y, fs, WHITE, sx, sy, 1.0);
            x += cvars.hud_scoreboard_width_name;
            render_text_with_shadow(&cvars, "Kills", x, y, fs, WHITE, sx, sy, 1.0);
            x += cvars.hud_scoreboard_width_kills;
            render_text_with_shadow(&cvars, "Deaths", x, y, fs, WHITE, sx, sy, 1.0);
            x += cvars.hud_scoreboard_width_deaths;
            render_text_with_shadow(&cvars, "Points", x, y, fs, WHITE, sx, sy, 1.0);

            y += cvars.hud_scoreboard_line_height as f32;

            for (player_handle, points) in player_points {
                let color = if player_handle == client.player_handle {
                    WHITE
                } else {
                    Color::new(0.8, 0.8, 0.8, 1.0)
                };
                let player = &server.gs.players[player_handle];
                let name = &player.name;
                let kills = &player.score.kills.to_string();
                let deaths = &player.score.deaths.to_string();
                let points = &points.to_string();

                x = x_start;
                render_text_with_shadow(&cvars, name, x, y, fs, color, sx, sy, 1.0);
                x += cvars.hud_scoreboard_width_name;
                render_text_with_shadow(&cvars, kills, x, y, fs, color, sx, sy, 1.0);
                x += cvars.hud_scoreboard_width_kills;
                render_text_with_shadow(&cvars, deaths, x, y, fs, color, sx, sy, 1.0);
                x += cvars.hud_scoreboard_width_deaths;
                render_text_with_shadow(&cvars, points, x, y, fs, color, sx, sy, 1.0);

                y += cvars.hud_scoreboard_line_height as f32;
            }
        }

        // Pause
        if server.paused {
            let paused_size = measure_text("PAUSED", None, cvars.hud_pause_font_size as u16, 1.0);
            // LATER remove cvars.hud_pause_x/y if raw_canvas removed
            render_text_with_shadow(
                &cvars,
                "PAUSED",
                (view_size.x as f32 - paused_size.width) / 2.0,
                (view_size.y as f32 - paused_size.height) / 2.0,
                cvars.hud_pause_font_size,
                RED,
                cvars.hud_pause_shadow_mq_x,
                cvars.hud_pause_shadow_mq_y,
                1.0,
            );
        }

        // Draw screen space debug info:

        // Draw FPS
        if cvars.d_fps {
            let fps_pos = hud_pos(screen_size, cvars.d_fps_x, cvars.d_fps_y);
            render_text_with_shadow(
                &cvars,
                &format!(
                    "update FPS: {:.1}   gamelogic FPS: {:.1}   render FPS: {:.1}",
                    server.update_fps.get_fps(),
                    server.gamelogic_fps.get_fps(),
                    client.render_fps.get_fps()
                ),
                fps_pos.x - 120.0, // LATER remove the offset after finding a decent font
                fps_pos.y,
                16.0,
                RED,
                1.0,
                1.0,
                0.5,
            );
        }

        // Draw perf info
        if cvars.d_draw && cvars.d_draw_perf {
            render_text_with_shadow(
                &cvars,
                &format!("last {} frames (in ms):", cvars.d_timing_samples),
                screen_size.x as f32 - 280.0,
                screen_size.y as f32 - 105.0,
                16.0,
                RED,
                1.0,
                1.0,
                0.5,
            );
            if let Some((avg, max)) = server.update_durations.get_stats() {
                let text = format!("update avg: {:.1}, max: {:.1}", avg * 1000.0, max * 1000.0);
                render_text_with_shadow(
                    &cvars,
                    &text,
                    screen_size.x as f32 - 280.0,
                    screen_size.y as f32 - 90.0,
                    16.0,
                    RED,
                    1.0,
                    1.0,
                    0.5,
                );
            }
            if let Some((avg, max)) = server.gamelogic_durations.get_stats() {
                let text = format!(
                    "gamelogic avg: {:.1}, max: {:.1}",
                    avg * 1000.0,
                    max * 1000.0
                );
                render_text_with_shadow(
                    &cvars,
                    &text,
                    screen_size.x as f32 - 280.0,
                    screen_size.y as f32 - 75.0,
                    16.0,
                    RED,
                    1.0,
                    1.0,
                    0.5,
                );
            }
            if let Some((avg, max)) = client.render_cmds_durations.get_stats() {
                let text = format!(
                    "render cmds avg: {:.1}, max: {:.1}",
                    avg * 1000.0,
                    max * 1000.0
                );
                render_text_with_shadow(
                    &cvars,
                    &text,
                    screen_size.x as f32 - 280.0,
                    screen_size.y as f32 - 60.0,
                    16.0,
                    RED,
                    1.0,
                    1.0,
                    0.5,
                );
            }
            if let Some((avg, max)) = client.rest_durations.get_stats() {
                let text = format!("rest avg: {:.1}, max: {:.1}", avg * 1000.0, max * 1000.0);
                render_text_with_shadow(
                    &cvars,
                    &text,
                    screen_size.x as f32 - 280.0,
                    screen_size.y as f32 - 45.0,
                    16.0,
                    RED,
                    1.0,
                    1.0,
                    0.5,
                );
            }
        }

        // Draw world debug text
        DEBUG_TEXTS_WORLD.with(|texts| {
            let texts = texts.borrow();
            if cvars.d_draw && cvars.d_draw_world_text {
                for text in texts.iter() {
                    let scr_pos = text.pos + camera_offset;
                    if cull(scr_pos) {
                        // Technically the text can be so long
                        // that it's culled overzealously but meh, perf is more important.
                        continue;
                    }

                    render_text_with_shadow(
                        &cvars,
                        &text.msg,
                        scr_pos.x as f32,
                        scr_pos.y as f32,
                        16.0,
                        RED,
                        1.0,
                        1.0,
                        0.5,
                    );
                }
            }
        });

        // Draw debug text
        let mut y = 25.0;
        DEBUG_TEXTS.with(|texts| {
            let texts = texts.borrow();
            if cvars.d_draw && cvars.d_draw_text {
                for text in texts.iter() {
                    render_text_with_shadow(&cvars, text, 20.0, y as f32, 16.0, RED, 1.0, 1.0, 0.5);
                    y += cvars.d_draw_text_line_height;
                }
            }
        });

        let end = get_time();
        client
            .render_cmds_durations
            .add(cvars.d_timing_samples, end - start);

        next_frame().await;

        let real_end = get_time();
        client
            .rest_durations
            .add(cvars.d_timing_samples, real_end - end);
    }
}

/// Place the image's *center* at `scr_pos`,
/// rotate it clockwise by `angle`.
///
/// See Vec2f for more about the coord system and rotations.
fn render_img_center(img: Texture2D, pos: Vec2f, angle: f64) {
    draw_texture_ex(
        img,
        pos.x as f32 - img.width() / 2.0,
        pos.y as f32 - img.height() / 2.0,
        WHITE,
        DrawTextureParams {
            rotation: angle as f32,
            ..Default::default()
        },
    );
}

/// Place the `img`'s *center of rotation* at `scr_pos`,
/// rotate it clockwise by `angle`.
/// The center of rotation is `img`'s center + `offset`.
///
/// See Vec2f for more about the coord system and rotations.
fn render_img_offset(img: Texture2D, pos: Vec2f, angle: f64, offset: Vec2f) {
    draw_texture_ex(
        img,
        // This is effectively `pos - (offset + half_size)`, just written differently.
        (pos.x - offset.x) as f32 - img.width() / 2.0,
        (pos.y - offset.y) as f32 - img.height() / 2.0,
        WHITE,
        DrawTextureParams {
            rotation: angle as f32,
            pivot: Some(Vec2::new(pos.x as f32, pos.y as f32)),
            ..Default::default()
        },
    );
}

fn render_tile(img: Texture2D, x: f64, y: f64, angle: f64) {
    draw_texture_ex(
        img,
        x as f32,
        y as f32,
        WHITE,
        DrawTextureParams {
            rotation: angle as f32,
            ..Default::default()
        },
    );
}

fn render_line(src: Vec2f, dest: Vec2f, thickness: f64, color: Color) {
    macroquad::shapes::draw_line(
        src.x as f32,
        src.y as f32,
        dest.x as f32,
        dest.y as f32,
        thickness as f32,
        color,
    );
}

#[allow(clippy::too_many_arguments)]
fn render_text_with_shadow(
    cvars: &Cvars,
    text: &str,
    mut x: f32,
    mut y: f32,
    font_size: f64,
    color: Color,
    shadow_offset_x: f32,
    shadow_offset_y: f32,
    shadow_alpha: f64,
) {
    if cvars.r_text_integer_positions {
        x = x.floor();
        y = y.floor();
    }
    if shadow_offset_x != 0.0 || shadow_offset_y != 0.0 {
        draw_text(
            &text,
            x + shadow_offset_x,
            y + shadow_offset_y,
            font_size as f32,
            Color::new(0.0, 0.0, 0.0, shadow_alpha as f32),
        );
    }
    draw_text(&text, x, y, font_size as f32, color);
}

/// If x or y are negative, count them from the right or bottom respectively.
/// Useful to make HUD config cvars work for any screen/view size.
fn hud_pos(size: Vec2f, mut x: f64, mut y: f64) -> Vec2 {
    if x < 0.0 {
        x += size.x;
    }
    if y < 0.0 {
        y += size.y;
    }
    Vec2::new(x as f32, y as f32)
}

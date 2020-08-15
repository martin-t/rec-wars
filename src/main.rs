// TODO MVP:
// [x] get simple.wasm working
// [x] compile rust to WASM, load that
// [x] canvas
// [x] render background
// [x] move around, check perf
//      https://github.com/mrdoob/stats.js/
//      [ ] make explosion sprite smaller
//      https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API/Tutorial/Optimizing_canvas
//          [x] round to whole pixels (orig RW does it)
//          [ ] webgl
// [x] pick a math lib, don't overthink it
// [ ] engine? rendering crate? nice if native and WASM backends
//      [x] check https://arewegameyet.rs/ecosystem/engines/
//      [x] check https://github.com/dasifefe/rust-game-development-frameworks
//      engines
//          godot
//          https://crates.io/crates/bevy - no WASM
//          https://crates.io/crates/amethyst - no menton of WASM
//          https://crates.io/crates/piston - no menton of WASM
//          https://crates.io/crates/ggez - WASM in progress
//          https://crates.io/crates/quicksilver
//              all 3 OSes + WASM supported
//              no audio?
//              has game showcase in readme (mostly bad)
//          macroquad - no docs?
//      rendering only
//          https://crates.io/crates/luminance - mentions webgl/canvas
// [ ] solve tearing - maybe only align to pixels at high speeds?
// [x] image fails to draw the first time after opening browser
//      bug in python server maybe? - doesn't happen with rhino
// [ ] server framerate when minimized - https://developer.mozilla.org/en-US/docs/Web/API/Window/setImmediate#Notes
//      postMessage / MessageChannel / https://github.com/YuzuJS/setImmediate polyfill
// [ ] extract/screenshot/record assets from RecWar or find alternatives
//      [ ] images
//          [ ] weapon icons
//          [ ] weapons
//          [ ] vehicles + skins
//          [ ] cow, stolen effects
//          [ ] wrecks
//      [ ] sounds
//          [ ] weapons, explosions, self destruct
//          [ ] cow
//          [ ] engine noise
//              [ ] how does it change with speed?
// [ ] CI - GH actions / travis, mirror to GL???
// [x] load maps
// [x] cvars
//      https://crates.io/crates/cvar
//          no tab completion in js console
//          https://crates.io/crates/amethyst-console - uses cvar and imgui
// [ ] render vehicles
//      [x] basic tank
//      [ ] skins, colors
//          canvas imageData?
//      [ ] vehicle types
// [x] explosions
//      [ ] sizes
// [ ] movement
//      [ ] reverse steering
//          https://engineeringdotnet.blogspot.com/2010/04/simple-2d-car-physics-in-games.html
//          https://www.asawicki.info/Mirror/Car%20Physics%20for%20Games/Car%20Physics%20for%20Games.html
// [ ] collision detection with proper traces
//      [ ] walls
//      [ ] vehicles
// [ ] physics
//      [ ] surfaces
//      [ ] pushing wrecks
// [ ] hp/health, wrecks (vehicle turned into wreck immediately on hit)
//      [ ] configurable
// [ ] weapons
//      [ ] MG - more on hovercraft
//      [ ] rail - from chassis on hummer
//      [ ] CB
//          [ ] explosions happen on walls, just partially obscured
//          [ ] reflect off map edge
//      [ ] rockets - do they accell?
//      [ ] homing missile
//      [ ] guided missile
//      [ ] BFG - delay? speed change? range? random dir when dead? explosion range (through wall)?
//          explosion animation goes backwards
// [ ] mines
//      [ ] not within radius of base/cow
// [ ] shadows
// [ ] self destruct - bigger exposion, countdown, sounds
//      works through walls
// [ ] UI
//      requirements
//          minimum: select mode, map, bots, start game
//          server list? quick join? start server?
//          tank customization (only pattern, colors, name - vehicle type should be changeable in-game)
//          binds (show keyboard image?)
//      maybe https://www.sitepoint.com/css-layouts-floats-flexbox-grid/
// [ ] fullscreen
// [ ] FFA
// [ ] TW
// [ ] CTC
//      [ ] cow movement
// [ ] bot AI
//      [ ] pathfinding - custom / from soko solver / crate?
//          https://old.reddit.com/r/rust_gamedev/comments/hr7m3j/which_lib_do_you_use_for_pathfinding_in_you_games/
//      [ ] roles / commands
// [ ] decent default binds (2x for splitscreen)
// [ ] hosting
//      GH pages? - needs public repo first
//      domain? SSL?
//      [ ] GH Social preview
//      master server? cloudflare, heroku, google app engine?
//          needs to be stateful
//      if dedicated servers, need at least 2 - EU and US
// [x] icon
// [ ] network
//      https://arewegameyet.rs/ecosystem/networking/
//          https://crates.io/crates/quinn - what is QUIC?
//          https://crates.io/crates/laminar - mentions only UDP, not WASM
//      WASM doesn't allow UDP sockets
//          https://gafferongames.com/post/why_cant_i_send_udp_packets_from_a_browser/
//          https://www.html5rocks.com/en/tutorials/webrtc/datachannels/
//          TCP
//          webRTC
//          crazy idea: multiple TCP streams, rotate through them
//      https://arewegameyet.rs/ecosystem/networking/ or custom?
//      [ ] master server hosting
//      [ ] prediction / reconciliation
//          braid / jonathan blow - fps independent
//          https://github.com/pond3r/ggpo/tree/master/doc
// [ ] chat
// [ ] voting
//      [ ] mode
//      [ ] map
//      [ ] cvars
// [ ] analytics
//      https://simpleanalytics.com/sandspiel.club - paid only
// [ ] FAQ - stuttering/tearing due to compositor - Alt+Shift+f12 - somehow this doesn't work anymore
// [ ] easter eggs
//      BFG can shoot through walls touching on the corner (well, somewhat)
//          detect when doing multiple times and print a message
//              private?
//              server say? (lol no collision detection?)
//      BFG does a tiny bit of dmg when hitting a wall with a tank on the other side
//          probably BFG briefly enters wall before collision is detected and does proximity dmg
//      server say for 0 deaths
// nice to have:
// [ ] make driving feel more real - simulate terrain unevenness? mild speed/angle changes?
// [ ] map editor - sharing maps, voting, recommended mode / number of bots
//      [ ] bots say hi/gg/sry/n1
// [ ] log of past games (to show activity even if nobody currently online)
// [ ] splitscreen
// [ ] figure out what webpack is and how to create a static site
//      probably best template: https://github.com/rustwasm/rust-webpack-template
// [ ] native binary
//      [ ] make parsing return errors instead of crashing
// [x] pause, variable speed
// [ ] frame debug mode - only render gamelogic frames, no interpolation
// [ ] shield pickups

fn main() {}

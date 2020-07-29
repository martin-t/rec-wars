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
//          [ ] layer canvases - background vs entities
//          [ ] webgl
// [x] pick a math lib, don't overthink it
// [ ] engine? rendering crate? nice if native and WASM backends
//      https://crates.io/crates/ggez - WASM in progress
//      https://crates.io/crates/luminance - mentions webgl/canvas
//      https://crates.io/crates/piston
//      https://crates.io/crates/quicksilver - all 3 OSes + WASM supported
//      miniquad, macroquad, godot
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
// [ ] load maps
// [ ] render tank, explosions
// [ ] movement, collision detection, physics
// [ ] weapons
// [ ] mines
//      [ ] not within radius of base/cow
// [ ] shadows
// [ ] self destruct - bigger exposion, countdown, sounds
// [ ] UI to select mode, map, bots, start game
// [ ] fullscreen?
// [ ] FFA, TW, CTC
// [ ] bot AI
//      [ ] pathfinding - custom / from soko solver / crate?
//          https://old.reddit.com/r/rust_gamedev/comments/hr7m3j/which_lib_do_you_use_for_pathfinding_in_you_games/
//      [ ] roles / commands
// [ ] hosting
//      GH pages? - needs public repo first
//      domain? SSL?
//      [ ] GH Social preview
//      master server? cloudflare, heroku, google app engine?
//          needs to be stateful
//      if dedicated servers, need at least 2 - EU and US
// [ ] icon
// [ ] network - UDP tunneling?
//      https://arewegameyet.rs/ecosystem/networking/ or custom?
//      [ ] master server hosting
//      [ ] prediction / reconciliation
//          braid / jonathan blow - fps independent
//          https://github.com/pond3r/ggpo/tree/master/doc
// [ ] cvars
//      [ ] voting
// [ ] analytics
//      https://simpleanalytics.com/sandspiel.club - paid only
// [ ] FAQ - stuttering/tearing due to compositor - Alt+Shift+f12 - somehow this doesn't work anymore
// [ ] easter eggs
//      BFG can shoot through walls touching on the corner (well, somewhat)
//          detect when doing multiple times and print a message
//              private?
//              server say? (lol no collision detection?)
// nice to have:
// [ ] map editor - sharing maps, voting, recommended mode / number of bots
// [ ] chat
//      [ ] bots say hi/gg/sry/n1
// [ ] log of past games (to show activity even if nobody currently online)
// [ ] splitscreen
// [ ] figure out what webpack is and how to create a static site
//      probably best template: https://github.com/rustwasm/rust-webpack-template

fn main() {}

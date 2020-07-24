use std::fs;

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
// [ ] engine? rendering crate? nice if native and WASM backends
//      https://crates.io/crates/ggez - WASM in progress
//      https://crates.io/crates/luminance - no mention os WASM
//      https://crates.io/crates/piston
//      https://crates.io/crates/quicksilver - all 3 OSes + WASM supported
//      miniquad, macroquad, godot
// [x] image fails to draw the first time after opening browser
//      bug in python server maybe? - doesn't happen with rhino
// [ ] server framerate when minimized - https://developer.mozilla.org/en-US/docs/Web/API/Window/setImmediate#Notes
//      postMessage / MessageChannel / https://github.com/YuzuJS/setImmediate polyfill
// [ ] extract assets from RecWar or find alternatives
//      [ ] images: weapon icons, tank skins, cow - maybe just screenshot?
//      [ ] sounds: weapons, explosions, cow, engine noise
// [ ] CI - GH actions / travis, mirror to GL???
// [ ] render tank, explosions
// [ ] movement, collision detection, physics
// [ ] weapons
// [ ] mines
//      [ ] not within radius of base/cow
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
// nice to have:
// [ ] map editor - sharing maps, voting, recommended mode / number of bots
// [ ] chat
//      [ ] bots say hi/gg/sry/n1
// [ ] log of past games (to show activity even if nobody currently online)
// [ ] splitscreen
// [ ] figure out what webpack is and how to create a static site

#[derive(Debug, Clone)]
struct Texture {
    name: String,
    a: i32,
    b: f32,
    c: f32,
}

impl Texture {
    fn new(name: String, a: i32, b: f32, c: f32) -> Self {
        Self { name, a, b, c }
    }
}

fn main() {
    let textures = load_textures();
    dbg!(&textures);
    let map = load_map("maps/Corners (4).map");
    dbg!(&map);
}

fn load_map(path: &str) -> Vec<Vec<usize>> {
    let text = fs::read_to_string(path).unwrap();
    dbg!(&text);
    text.split_terminator("\r\n")
        .map(|line| line.split(" ").map(|tile| tile.parse().unwrap()).collect())
        .collect()
}

fn load_textures() -> Vec<Texture> {
    let texture_list = fs::read_to_string("TextureList.txt").unwrap();
    dbg!(&texture_list);
    // TODO handle both CRLF and LF properly
    texture_list
        .split_terminator("\r\n")
        .map(|line| {
            dbg!(line);
            let mut parts = line.split(" ");
            let name = parts.next().unwrap();
            let a = parts.next().unwrap().parse().unwrap();
            let b = parts.next().unwrap().parse().unwrap();
            let c = parts.next().unwrap().parse().unwrap();
            Texture::new(name.to_owned(), a, b, c)
        })
        .collect()
}

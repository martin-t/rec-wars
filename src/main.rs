use std::fs;

// TODO MVP:
// [x] get simple.wasm working
// [ ] compile rust to WASM, load that
// [x] canvas
// [x] render background
// [ ] move around, check perf
//      https://github.com/mrdoob/stats.js/
// [ ] engine? rendering crate? nice if native and WASM backends
//      https://crates.io/crates/ggez - WASM in progress
//      https://crates.io/crates/luminance - no mention os WASM
//      https://crates.io/crates/piston
//      https://crates.io/crates/quicksilver - all 3 OSes + WASM supported
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
// [ ] UI to select mode, map, bots, start game
// [ ] fullscreen?
// [ ] FFA, TW, CTC
// [ ] bot AI
//      [ ] roles / commands
// [ ] hosting - GH pages? domain? SSL?
// [ ] network - UDP tunneling?
//      https://arewegameyet.rs/ecosystem/networking/ or custom?
//      [ ] master server hosting
// [ ] cvars
//      [ ] voting
// [ ] analytics - e.g. https://simpleanalytics.com/sandspiel.club
// nice to have:
// [ ] map editor - sharing maps, voting, recommended mode / number of bots
// [ ] chat
//      [ ] bots say hi/gg/sry/n1
// [ ] log of past games (to show activity even if nobody currently online)

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

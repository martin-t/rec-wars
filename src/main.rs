use std::fs;

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

/// The value / 4 is index into texture_list.txt.
/// The value % 4 is the rotation TODO
pub fn load_map(text: &str) -> Vec<Vec<usize>> {
    // TODO handle both CRLF and LF properly
    text.split_terminator("\r\n")
        .map(|line| line.split(" ").map(|tile| tile.parse().unwrap()).collect())
        .collect()
}

#[derive(Debug, Clone)]
pub struct Texture {
    name: String,
    // TODO what do these mean?
    a: i32,
    b: f32,
    c: f32,
}

impl Texture {
    fn new(name: String, a: i32, b: f32, c: f32) -> Self {
        Self { name, a, b, c }
    }
}

pub fn load_textures(text: &str) -> Vec<Texture> {
    // TODO handle both CRLF and LF properly OR use cvars instead
    // if using cvars, update load_map docs
    text.split_terminator("\r\n")
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    #[test]
    fn test_loading_maps() {
        let mut cnt = 0;
        for entry in fs::read_dir("maps").unwrap() {
            let entry = entry.unwrap();
            let text = fs::read_to_string(entry.path()).unwrap();
            let map = load_map(&text);
            assert_ne!(map.len(), 0);
            assert_ne!(map[0].len(), 0);
            cnt += 1;
        }
        assert_ne!(cnt, 0);
    }

    #[test]
    fn test_loading_textures() {
        let textures = fs::read_to_string("assets/texture_list.txt").unwrap();
        assert_ne!(textures.len(), 0);
    }
}

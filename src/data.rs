use std::ops::Index;

use approx::AbsDiffEq;

use vek::Clamp;
use vek::Vec2;

pub type Vec2f = Vec2<f64>;
pub type Vec2u = Vec2<usize>;

pub const TILE_SIZE: f64 = 64.0;

pub fn load_map(text: &str) -> Map {
    // TODO handle both CRLF and LF properly
    let tiles = text
        .split_terminator("\r\n")
        .map(|line| {
            line.split(" ")
                .map(|tile| {
                    let val: usize = tile.parse().unwrap();
                    // TODO to rad
                    Tile {
                        surface: val / 4,
                        rotation: (val % 4) as f64,
                    }
                })
                .collect()
        })
        .collect();
    Map::new(tiles)
}

#[derive(Debug, Clone)]
pub struct Map {
    tiles: Vec<Vec<Tile>>,
}

impl Map {
    fn new(tiles: Vec<Vec<Tile>>) -> Self {
        Map { tiles }
    }

    pub fn height(&self) -> usize {
        self.tiles.len()
    }

    pub fn width(&self) -> usize {
        self.tiles[0].len()
    }

    /// Width, height / cols, rows / x, y
    pub fn size(&self) -> Vec2u {
        Vec2u::new(self.width(), self.height())
    }

    /// Lowest possible coordinates / top left
    ///
    /// Currently for simplicity always 0,0.
    /// Might change in the future e.g. if symmetry is easier with 0,0 in the center.
    pub fn mins(&self) -> Vec2f {
        Vec2::new(0.0, 0.0)
    }

    /// Highest possible coordinates / bottom right
    pub fn maxs(&self) -> Vec2f {
        self.size().as_() * TILE_SIZE
    }

    /// Col is x, row is y
    pub fn col_row(&self, c: usize, r: usize) -> &Tile {
        &self[Vec2::new(c, r)]
    }

    /// Converts world coords into tile position and offset within it.
    ///
    /// The returned index will always be within bounds.
    pub fn tile_pos(&self, pos: Vec2f) -> TilePos {
        let epsilon = self.maxs() * Vec2f::default_epsilon();
        let pos = pos.clamped(self.mins(), self.maxs() - epsilon);
        let index = (pos / TILE_SIZE).as_();
        let offset = pos % TILE_SIZE;
        TilePos { index, offset }
    }
}

impl Index<Vec2u> for Map {
    type Output = Tile;
    fn index(&self, index: Vec2u) -> &Self::Output {
        &self.tiles[index.y][index.x]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TilePos {
    /// Position of the tile in the map
    pub index: Vec2u,
    /// Offset inside the tile
    pub offset: Vec2f,
}

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    /// Index into texture_list.txt
    pub surface: usize,
    /// Rotation counterclockwise
    pub rotation: f64,
}

#[derive(Debug, Clone)]
pub struct Surface {
    pub name: String,
    pub kind: Kind,
    /// Seems to affect both turning and accellaration
    pub friction: f32,
    /// Maybe a multiplier for speed
    pub speed: f32,
}

impl Surface {
    fn new(name: String, kind: Kind, friction: f32, speed: f32) -> Self {
        Self {
            name,
            kind,
            friction,
            speed,
        }
    }
}

/// Reverse engineered by modifying TextureList.txt and seeing what happens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Normal = 0,
    Spawn = 1,
    Wall = 2,
    /// Vehicles on it spawn particles on their sides
    Water = 3,
    /// I don't see any effect
    Snow = 4,
    Base = 5,
}

impl Kind {
    pub fn new(n: i32) -> Option<Self> {
        match n {
            0 => Some(Kind::Normal),
            1 => Some(Kind::Spawn),
            2 => Some(Kind::Wall),
            3 => Some(Kind::Water),
            4 => Some(Kind::Snow),
            5 => Some(Kind::Base),
            _ => None,
        }
    }
}

pub fn load_textures(text: &str) -> Vec<Surface> {
    // TODO handle both CRLF and LF properly OR use cvars instead
    // if using cvars, update load_map docs
    text.split_terminator("\r\n")
        .map(|line| {
            dbg!(line);
            let mut parts = line.split(" ");
            let name = parts.next().unwrap();
            let kind_num = parts.next().unwrap().parse().unwrap();
            let friction = parts.next().unwrap().parse().unwrap();
            let speed = parts.next().unwrap().parse().unwrap();

            let kind = Kind::new(kind_num).unwrap();
            Surface::new(name.to_owned(), kind, friction, speed)
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
            assert_ne!(map.width(), 0);
            assert_ne!(map.height(), 0);
            cnt += 1;
        }
        assert_ne!(cnt, 0);
    }

    #[test]
    fn test_loading_texture_list() {
        let text = fs::read_to_string("assets/texture_list.txt").unwrap();
        let textures = load_textures(&text);
        assert_ne!(textures.len(), 0);
    }
}

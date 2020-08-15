use std::f64::consts::PI;
use std::ops::Index;

use approx::AbsDiffEq;

use vek::Clamp;
use vek::Vec2;

/// Position in world or screen space.
///
/// ### Coord system
/// `x` is right, `y` is down - origin is top-left.
/// This is to make world and screen coords behave the same
/// (although I believe it's more common for world coords to start in bottom-left so that `y` is up).
/// The result of having `y` down is that the unit circle in mirrored around the X axis.
/// As a result, **angles are clockwise**, in radians and 0 is pointing right.
pub type Vec2f = Vec2<f64>;

/// Position of a tile in the map.
///
/// To avoid confusion with world positions,
/// it's sometimes referred to as tile index since it's a pair of indices.
/// `x` is column, `y` is row to match the order of `Vec2f`.
pub type Vec2u = Vec2<usize>;

pub const TILE_SIZE: f64 = 64.0;

/// A rectangular tile based map with origin in the top-left corner.
#[derive(Debug, Clone)]
pub struct Map {
    tiles: Vec<Vec<Tile>>,
    spawns: Vec<Vec2u>,
    bases: Vec<Vec2u>,
}

impl Map {
    fn new(tiles: Vec<Vec<Tile>>, surfaces: &[Surface]) -> Self {
        let mut spawns = Vec::new();
        let mut bases = Vec::new();
        for (r, row) in tiles.iter().enumerate() {
            for (c, tile) in row.iter().enumerate() {
                let kind = surfaces[tile.surface].kind;
                if kind == Kind::Spawn {
                    spawns.push(Vec2u::new(c, r));
                } else if kind == Kind::Base {
                    bases.push(Vec2u::new(c, r));
                }
            }
        }
        Map {
            tiles,
            spawns,
            bases,
        }
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
        // NOTE: if changing mins to be negative, check all uses of the modulo operator
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
        // This only works properly with positive numbers but it's ok
        // as long as top left of the map is (0.0, 0.0).
        let offset = pos % TILE_SIZE;
        TilePos { index, offset }
    }

    pub fn tile_center(&self, tile_index: Vec2u) -> Vec2f {
        tile_index.as_() * TILE_SIZE + TILE_SIZE / 2.0
    }

    pub fn spawns(&self) -> &Vec<Vec2u> {
        &self.spawns
    }

    #[allow(unused)] // TODO remove after CTC works
    pub fn bases(&self) -> &Vec<Vec2u> {
        &self.bases
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
    /// Rotation in radians - see Vec2f for how the coord system and angles work.
    pub angle: f64,
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

/// Special behavior of some tiles.
///
/// Reverse engineered by modifying RecWar's TextureList.txt and seeing what happens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// No special behavior beyond the normal surface properties
    Normal = 0,
    /// Vehicles spawn on it
    Spawn = 1,
    /// Solid - can't move through it, most weapons can't shoot through it
    Wall = 2,
    /// Vehicles on it spawn particles on their sides
    Water = 3,
    /// I don't see any effect
    Snow = 4,
    /// Base for Capture the Cow
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

pub fn load_map(text: &str, surfaces: &[Surface]) -> Map {
    // TODO handle both CRLF and LF properly
    let tiles = text
        .split_terminator("\r\n")
        .map(|line| {
            line.split(" ")
                .map(|tile| {
                    let val: usize = tile.parse().unwrap();
                    // rotation is number of turns counterclockwise
                    // angle is clockwise (see Vec2f for coord system explanation)
                    // g_spawn: rotation - angle - meaning
                    // 0    0           right
                    // 1    -1/2*PI     up
                    // 2    -PI         left
                    // 3    -3/2*PI     down
                    let rotation = val % 4;
                    Tile {
                        surface: val / 4,
                        angle: rotation as f64 * -PI / 2.0,
                    }
                })
                .collect()
        })
        .collect();
    Map::new(tiles, surfaces)
}

pub fn load_tex_list(text: &str) -> Vec<Surface> {
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
    fn test_loading_tex_list() {
        let text = fs::read_to_string("assets/texture_list.txt").unwrap();
        let surfaces = load_tex_list(&text);
        assert_ne!(surfaces.len(), 0);
    }

    #[test]
    fn test_loading_maps() {
        let mut cnt = 0;

        let tex_list_text = fs::read_to_string("assets/texture_list.txt").unwrap();
        let surfaces = load_tex_list(&tex_list_text);
        for entry in fs::read_dir("maps").unwrap() {
            let entry = entry.unwrap();
            let map_text = fs::read_to_string(entry.path()).unwrap();
            let map = load_map(&map_text, &surfaces);
            assert_ne!(map.width(), 0);
            assert_ne!(map.height(), 0);
            cnt += 1;
        }
        assert_ne!(cnt, 0);
    }

    #[test]
    fn test_map_a_simple_plan() {
        let tex_list_text = fs::read_to_string("assets/texture_list.txt").unwrap();
        let surfaces = load_tex_list(&tex_list_text);
        let map_text = fs::read_to_string("maps/A simple plan (2).map").unwrap();
        let map = load_map(&map_text, &surfaces);
        assert_eq!(map.width(), 55);
        assert_eq!(map.height(), 23);
        assert_eq!(map.size(), Vec2u::new(55, 23));
        assert_eq!(map.mins(), Vec2f::new(0.0, 0.0));
        assert_eq!(map.maxs(), Vec2f::new(55.0 * 64.0, 23.0 * 64.0));

        assert_eq!(map.tile_center(Vec2u::new(0, 0)), Vec2f::new(32.0, 32.0));
        assert_eq!(map.tile_center(Vec2u::new(1, 0)), Vec2f::new(96.0, 32.0));
        assert_eq!(map.tile_center(Vec2u::new(0, 1)), Vec2f::new(32.0, 96.0));

        assert_eq!(map.spawns().len(), 24);
        assert_eq!(map.spawns()[0], Vec2u::new(9, 3));
        assert_eq!(map.bases().len(), 2);
        assert_eq!(map.bases()[0], Vec2u::new(10, 11));
    }
}

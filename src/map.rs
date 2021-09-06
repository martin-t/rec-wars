//! Map data (everything static during a match) and coordinate system

use std::f64::consts::PI;
use std::ops::Index;

use enumn::N;
use rand::{prelude::SmallRng, Rng};
use vek::{approx::AbsDiffEq, Clamp, Mat2, Vec2};

/// Position in world or screen space.
///
/// ### Coord system
/// `x` is right, `y` is down - origin is top-left.
/// This is to make world and screen coords behave the same
/// (although I believe it's more common for world coords to start in bottom-left so that `y` is up).
/// The result of having `y` down is that the unit circle in mirrored around the X axis.
/// As a result, **angles are clockwise**, in radians and 0 is pointing right.
pub type Vec2f = Vec2<f64>;

pub type Mat2f = Mat2<f64>;

/// Position of a tile in the map.
///
/// To avoid confusion with world positions,
/// it's sometimes referred to as tile index since it's a pair of indices.
/// `x` is column, `y` is row to match the order of `Vec2f`.
pub type Vec2u = Vec2<usize>;

pub trait VecExt {
    fn to_angle(self) -> f64;
}

impl VecExt for Vec2f {
    fn to_angle(self) -> f64 {
        self.y.atan2(self.x)
    }
}

pub trait F64Ext {
    /// Rotated unit vector
    fn to_vec2f(self) -> Vec2f;

    /// 2D rotation matrix
    fn to_mat2f(self) -> Mat2f;
}

impl F64Ext for f64 {
    fn to_vec2f(self) -> Vec2f {
        Vec2f::new(self.cos(), self.sin())
    }

    fn to_mat2f(self) -> Mat2f {
        Mat2f::rotation_z(self)
    }
}

pub const TILE_SIZE: f64 = 64.0;

/// A rectangular tile based map with origin in the top-left corner.
#[derive(Debug, Clone)]
pub struct Map {
    surfaces: Vec<Surface>,
    tiles: Vec<Vec<Tile>>,
    spawns: Vec<Vec2u>,
    bases: Vec<Vec2u>,
}

impl Map {
    fn new(tiles: Vec<Vec<Tile>>, surfaces: Vec<Surface>) -> Self {
        let mut spawns = Vec::new();
        let mut bases = Vec::new();
        for (r, row) in tiles.iter().enumerate() {
            for (c, tile) in row.iter().enumerate() {
                let kind = surfaces[tile.surface_index].kind;
                if kind == Kind::Spawn {
                    spawns.push(Vec2u::new(c, r));
                } else if kind == Kind::Base {
                    bases.push(Vec2u::new(c, r));
                }
            }
        }
        Map {
            surfaces,
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

    /// Returns (width, height) / (cols, rows) / (x, y)
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

    /// Returns tile at (c,r). Col is x, row is y
    pub fn col_row(&self, c: usize, r: usize) -> Tile {
        self[Vec2::new(c, r)]
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

    pub fn surface_of(&self, tile: Tile) -> &Surface {
        &self.surfaces[tile.surface_index]
    }

    pub fn surface_at_pos(&self, pos: Vec2f) -> &Surface {
        let tile_pos = self.tile_pos(pos);
        self.surface_at_index(tile_pos.index)
    }

    pub fn surface_at_index(&self, index: Vec2u) -> &Surface {
        let surface_index = self[index].surface_index;
        &self.surfaces[surface_index]
    }

    /// Is `pos` outside the map or inside a wall?
    pub fn is_wall(&self, pos: Vec2f) -> bool {
        if pos.x <= 0.0 {
            return true;
        }
        if pos.y <= 0.0 {
            return true;
        }
        let map_size = self.maxs();
        if pos.x >= map_size.x {
            return true;
        }
        if pos.y >= map_size.y {
            return true;
        }

        let kind = self.surface_at_pos(pos).kind;
        if kind == Kind::Wall {
            return true;
        }

        false
    }

    /// Find first wall collision when traveling from `begin` to `end`.
    /// The returned point is nudged slightly inside the wall.
    /// Area outside the map is considered wall.
    pub fn is_wall_trace(&self, begin: Vec2f, end: Vec2f) -> Option<Vec2f> {
        if self.is_wall(begin) {
            return Some(begin);
        }

        // similar to the simple, unoptimized version here:
        // https://playtechs.blogspot.com/2007/03/raytracing-on-grid.html

        let delta = end - begin;
        let nudge = delta.normalized() * 0.01;

        // Find closest vertical and horizontal intersections with the grid.
        let mut t_x;
        if delta.x == 0.0 {
            t_x = f64::INFINITY;
        } else {
            // Next intersection along the X axis (with a vertical line).
            let next_x = if delta.x > 0.0 {
                (begin.x / TILE_SIZE).ceil() * TILE_SIZE
            } else {
                (begin.x / TILE_SIZE).floor() * TILE_SIZE
            };

            // How far along the line are we?
            let travelled_x = next_x - begin.x;
            t_x = travelled_x / delta.x;
        }
        let mut t_y;
        if delta.y == 0.0 {
            t_y = f64::INFINITY;
        } else {
            // Next intersection along the X axis (with a vertical line).
            let next_y;
            if delta.y > 0.0 {
                next_y = (begin.y / TILE_SIZE).ceil() * TILE_SIZE
            } else {
                next_y = (begin.y / TILE_SIZE).floor() * TILE_SIZE
            };

            // How far along the line are we?
            let travelled_y = next_y - begin.y;
            t_y = travelled_y / delta.y;
        }

        // After finding the first intersection, the subsequent steps all have the same size.
        let t_step_x = TILE_SIZE / delta.x.abs();
        let t_step_y = TILE_SIZE / delta.y.abs();
        loop {
            let t;
            if t_x < t_y {
                t = t_x;
                t_x += t_step_x;
            } else {
                t = t_y;
                t_y += t_step_y;
            };
            if t > 1.0 {
                return None;
            }
            let intersection = begin + delta * t;
            let wall = intersection + nudge;
            if self.is_wall(wall) {
                return Some(wall);
            }
        }
    }

    pub fn spawns(&self) -> &Vec<Vec2u> {
        &self.spawns
    }

    // LATER remove all #[allow(dead_code)] here (or the fns if they turn out useless)

    #[allow(dead_code)]
    pub fn bases(&self) -> &Vec<Vec2u> {
        &self.bases
    }

    /// Returns (pos, angle).
    pub fn random_spawn(&self, rng: &mut SmallRng) -> (Vec2f, f64) {
        // TODO maps with no spawns (or even all walls)
        let i = rng.gen_range(0..self.spawns().len());
        let index = self.spawns()[i];
        let pos = self.tile_center(index);
        let angle = self[index].angle;
        (pos, angle)
    }

    /// Returns (pos, angle).
    pub fn random_nonwall(&self, rng: &mut SmallRng) -> (Vec2f, f64) {
        loop {
            let c = rng.gen_range(0..self.width());
            let r = rng.gen_range(0..self.height());
            let index = Vec2u::new(c, r);
            if self.surface_at_index(index).kind != Kind::Wall {
                let pos = self.tile_center(index);
                let angle = self[index].angle;
                return (pos, angle);
            }
        }
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
    pub surface_index: usize,
    /// Rotation in radians - see Vec2f for how the coord system and angles work.
    pub angle: f64,
}

#[derive(Debug, Clone)]
pub struct Surface {
    pub name: String,
    pub kind: Kind,
    /// Seems to affect both turning and acceleration
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
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, N)]
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

pub fn load_map(text: &str, surfaces: Vec<Surface>) -> Map {
    // TODO handle both CRLF and LF properly
    // TODO move to Map::new()?
    let tiles = text
        .split_terminator("\r\n")
        .map(|line| {
            line.split(' ')
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
                        surface_index: val / 4,
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
            let mut parts = line.split(' ');
            let name = parts.next().unwrap();
            let kind_num = parts.next().unwrap().parse().unwrap();
            let friction = parts.next().unwrap().parse().unwrap();
            let speed = parts.next().unwrap().parse().unwrap();

            let kind = Kind::n(kind_num).unwrap();
            Surface::new(name.to_owned(), kind, friction, speed)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    use walkdir::WalkDir;

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
        for entry in WalkDir::new("maps") {
            let entry = entry.unwrap();
            let is_map = entry.file_name().to_str().unwrap().ends_with(".map");
            if entry.file_type().is_dir() || !is_map {
                continue;
            }

            dbg!(entry.file_name());
            let map_text = fs::read_to_string(entry.path()).unwrap();
            let map = load_map(&map_text, surfaces.clone());
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
        let map = load_map(&map_text, surfaces);
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

    #[test]
    fn test_collisions_between() {
        let tex_list_text = fs::read_to_string("assets/texture_list.txt").unwrap();
        let surfaces = load_tex_list(&tex_list_text);
        let map_text = fs::read_to_string("maps/Corners (4).map").unwrap();
        let map = load_map(&map_text, surfaces);

        let outside = Vec2f::new(-50.0, -50.0);

        // These form a triangle which clips the edge of a wall
        let top_left = map.tile_center(Vec2u::new(0, 0));
        let bottom_left = map.tile_center(Vec2u::new(0, 3));
        let top_right = map.tile_center(Vec2u::new(3, 0));

        assert!(map.is_wall_trace(outside, outside).is_some());
        assert!(map.is_wall_trace(outside, top_left).is_some());
        assert!(map.is_wall_trace(top_left, outside).is_some());

        assert!(map.is_wall_trace(top_left, top_left).is_none());
        assert!(map.is_wall_trace(top_left, bottom_left).is_none());
        assert!(map.is_wall_trace(bottom_left, top_left).is_none());

        let up = Vec2f::new(0.0, -10.0);
        assert!(map.is_wall_trace(bottom_left, top_right + up).is_none());
        assert!(map.is_wall_trace(bottom_left, top_right - up).is_some());
    }
}

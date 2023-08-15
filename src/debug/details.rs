//! Implementation details of debugging tools.
//!
//! They have to be public because the macros use them
//! but in normal usage you should prefer the `dbg_*` macros
//! and other items from the parent mod.

use vek::Vec2;

use crate::{debug::DEBUG_SHAPES, prelude::*};

#[macro_export]
macro_rules! __println {
    () => {
        $crate::__println!("")
    };
    ($($t:tt)*) => {
        // Use info so it shows up by default in chromium, debug doesn't.
        macroquad::logging::info!($($t)*)
    }
}

/// Helper struct, use one of the `dbg_*!()` macros.
#[derive(Debug, Clone)]
pub struct WorldText {
    pub pos: Vec2f,
    pub msg: String,
}

impl WorldText {
    pub fn new(pos: Vec2f, msg: String) -> Self {
        Self { pos, msg }
    }
}

/// Helper struct, use one of the `dbg_*!()` macros.
#[derive(Debug, Clone)]
pub struct DebugShape {
    pub shape: Shape,
    /// Time left (decreases every frame)
    pub time: f64,
    pub color: Color,
}

/// Helper enum, use one of the `dbg_*!()` macros.
#[derive(Debug, Clone)]
pub enum Shape {
    Line { begin: Vec2f, end: Vec2f },
    Arrow { begin: Vec2f, dir: Vec2f },
    Cross { point: Vec2f },
    Rot { point: Vec2f, rot: f64, scale: f64 },
}

/// Helper function, prefer `dbg_line!()` instead.
pub fn debug_line(begin: Vec2f, end: Vec2f, time: f64, color: Color) {
    let shape = Shape::Line { begin, end };
    debug_shape(shape, time, color);
}

/// Helper function, prefer `dbg_arrow!()` instead.
pub fn debug_arrow(begin: Vec2f, dir: Vec2f, time: f64, color: Color) {
    let shape = Shape::Arrow { begin, dir };
    debug_shape(shape, time, color);
}

/// Helper function, prefer `dbg_cross!()` instead.
pub fn debug_cross(point: Vec2f, time: f64, color: Color) {
    let shape = Shape::Cross { point };
    debug_shape(shape, time, color);
}

/// Helper function, prefer `dbg_rot!()` instead.
pub fn debug_rot(point: Vec2f, rot: f64, time: f64, scale: f64) {
    let shape = Shape::Rot { point, rot, scale };
    // Color is not used
    debug_shape(shape, time, WHITE);
}

fn debug_shape(shape: Shape, time: f64, color: Color) {
    DEBUG_SHAPES.with(|shapes| {
        let shape = DebugShape { shape, time, color };
        shapes.borrow_mut().push(shape);
    });
}

impl DebugShape {
    pub fn to_lines(&self, cvars: &Cvars, lines: &mut UniqueLines) {
        match self.shape {
            Shape::Line { begin, end } => {
                if !cvars.d_draw_lines {
                    return;
                }

                lines.insert(begin, end, self.color);

                if cvars.d_draw_lines_ends_half_length > 0.0 {
                    let dir = (end - begin).normalized();
                    let perpendicular = v!(-dir.y, dir.x) * cvars.d_draw_lines_ends_half_length;
                    lines.insert(begin - perpendicular, begin + perpendicular, self.color);
                    lines.insert(end - perpendicular, end + perpendicular, self.color);
                }
            }
            Shape::Arrow { begin, dir } => {
                if !cvars.d_draw_arrows {
                    return;
                }

                let end = begin + dir;
                lines.insert(begin, end, self.color);

                let rot = dir.to_angle();
                let len = dir.magnitude();
                let up = UP.rotated_z(rot) * len;
                lines.insert(end, end + (-dir + up) * 0.25, self.color);
                lines.insert(end, end + (-dir - up) * 0.25, self.color);
            }
            Shape::Cross { point } => {
                if !cvars.d_draw_crosses {
                    return;
                }

                let dir1 = v!(1 1) * cvars.d_draw_crosses_half_len;
                let dir2 = v!(-1 1) * cvars.d_draw_crosses_half_len;
                lines.insert(point - dir1, point + dir1, self.color);
                lines.insert(point - dir2, point + dir2, self.color);

                if cvars.d_draw_crosses_line_from_origin {
                    lines.insert(Vec2f::zero(), point, self.color);
                }
            }
            Shape::Rot { point, rot, scale } => {
                if !cvars.d_draw_rots {
                    return;
                }

                let size = scale * cvars.d_draw_rots_size;

                lines.insert(point, point + (size * v!(1 0)).rotated_z(rot), RED);
                lines.insert(point, point + (size * v!(0 1)).rotated_z(rot), GREEN);
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct UniqueLines(pub FnvHashMap<(Vec2<u64>, Vec2<u64>), Line>);

impl UniqueLines {
    /// Insert the line into the hashmap, merging colors if a line already exists
    /// in the exact same place.
    fn insert(&mut self, begin: Vec2f, end: Vec2f, color: Color) {
        // It might be tempting to add a tiny bit of tolerance
        // so lines close enough get merged
        // but it would make it hard to notice cl/sv desyncs.
        // At least it should be off by default.
        let bits_begin = begin.map(|v| v.to_bits());
        let bits_end = end.map(|v| v.to_bits());

        self.0
            .entry((bits_begin, bits_end))
            .and_modify(|line| line.color = Color::from_vec(line.color.to_vec() + color.to_vec()))
            .or_insert(Line { begin, end, color });
    }
}

/// Colored line between two points.
#[derive(Clone, Debug)]
pub struct Line {
    /// Beginning of the line.
    pub begin: Vec2f,
    /// End of the line.
    pub end: Vec2f,
    /// Color of the line.
    pub color: Color,
}

#[cfg(test)]
pub const V1: Vec2f = v!(1 2);
#[cfg(test)]
pub const V2: Vec2f = v!(4 5);
#[cfg(test)]
#[macro_export]
macro_rules! r1 {
    () => {
        0.123
    };
}

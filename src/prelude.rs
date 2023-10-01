//! A bunch of QoL consts, macros, traits and functions
//! to make gamedev in Rust faster and saner.
//!
//! Should be imported in most files via
//! `use crate::prelude::*`.

// The whole point of this mod is to provide a bunch of stuff
// that may or may not be used but should be *available*.
#![allow(unused_imports)]
#![allow(dead_code)]

// Some private imports that are intentionally *not* re-exported.
use vek::{Mat2, Vec2};

// Public re-exports.
// Make the most commonly used types available without importing manually.
// Criteria for inclusion: used often and unlikely to collide.

pub use std::{
    collections::VecDeque,
    f64::consts::PI,
    fmt::{self, Debug, Display, Formatter},
    str::FromStr,
};

pub use fnv::{FnvHashMap, FnvHashSet};

// This doesn't increase incremental compile times in any measureble way.
// Keep it here so it can be used immediately without adding to Cargo.toml or importing first.
pub use inline_tweak::tweak;

// Intentionally not re-exporting `macroquad::prelude::*` here because:
//  - It contain a lot of stuff not neeced in RecWars.
//  - It causes collisions with rand because it has its own random implementation.
//  - It's only used in a handful of files and the rest of the gamecode doesn't need to know about macroquad.
pub use macroquad::color::{colors::*, Color};

pub use rand::prelude::*;
// `rng.sample(Normal::new(mean, std_dev))` gives exactly the same results as
// `rng.sample(StandardNormal) * std_dev + mean`.
// The latter sometimes requires type annotations.
pub use rand_distr::{Normal, StandardNormal, Uniform};
pub use rand_xoshiro::Xoshiro256PlusPlus;

pub use serde::{Deserialize, Serialize};

pub use strum::EnumCount;
pub use strum_macros::{EnumCount, FromRepr};

pub use thunderdome::{Arena, Index};

pub use vek::{Clamp, Lerp, Slerp, Wrap};

pub use crate::{
    assets::Assets,
    client::{Client, ClientGame},
    context::{ClientFrameCtx, FrameCtx, ServerFrameCtx},
    cvars::*,
    debug::{DbgIterator, SoftUnwrap},
    entities::*,
    game_state::*,
    input::*,
    map::Map,
    net_messages::*,
    server::{Server, ServerGame},
    utils::lerp_ranges,
    weapons,
};

// Visibility of macros by example works diffrently from normal items,
// they behave as if they were defined in the crate's root
// so we import it here to make it part of prelude.
pub use crate::v;

/// Shorthand for `Vec2::new()`.
///
/// Short name, no decimal point (casts to f64), no commas between numbers.
///
/// X, Y is **right, down**.
///
/// ---
///
/// The most common usecase is a constant vector with all coords as number literals,
/// e.g. `v!(-42 420.69)`. If you need arbitrary expressions
/// (e.g. `v!(-s.x, a + b)`), you need to use commas
/// because expressions can contain spaces so they wouldn't work as a separator.
///
/// LATER Check f64 represents the input value exactly, log warn if not, rate limit it.
///
/// # Usage
///
/// ```rust
/// v!(1 2)
/// ```
#[macro_export]
macro_rules! v {
    // Support for arbitrary expressions - requires commas.
    ($x:expr, $y:expr) => {{
        #[allow(trivial_numeric_casts)]
        let tmp = Vec2f::new($x as f64, $y as f64);
        tmp
    }};
    // The simple usecase - no commas.
    ($x:literal $y:literal) => {
        v!($x, $y)
    };
}

/// Position in world or screen space.
///
/// ### Coord system
///
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

pub const RIGHT: Vec2f = v!(1 0);
pub const DOWN: Vec2f = v!(0 1);
pub const LEFT: Vec2f = v!(-1 0);
pub const UP: Vec2f = v!(0 - 1);

pub trait Vec2fExt {
    fn to_angle(self) -> f64;
}

impl Vec2fExt for Vec2f {
    fn to_angle(self) -> f64 {
        // Normalize to 0..=360 deg
        self.y.atan2(self.x).rem_euclid(2.0 * PI)
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

pub trait ArenaExt {
    /// Collect the handles (`thunderdome::Index`) into a `Vec`.
    ///
    /// This is a workaround for borrowck limitations so we can
    /// iterate over the pool without keeping it borrowed.
    /// You can reborrow each iteration of the loop by indexing the arena using the handle
    /// and release the borrow if you need to pass the arena (or usually the whole frame context)
    /// into another function.
    ///
    /// This is inefficient and ideally should be avoided
    /// but contrary to everyone in Rust gamedev circles talking about performance,
    /// most games are not limited by how fast their gamelogic runs.
    /// When/if we have perf issues and profiling says this is the cause,
    /// then we can restructure the code to avoid it.
    /// Until then writing code faster is more important than writing faster code.
    fn collect_handles(&self) -> Vec<Index>;

    fn collect_indices(&self) -> Vec<u32>;

    /// Converts an index (slot in thunderdome) to a handle (index in thunderdome).
    fn slot_to_index(&self, slot: u32) -> Option<Index>;
}

impl<T> ArenaExt for Arena<T> {
    fn collect_handles(&self) -> Vec<Index> {
        self.iter().map(|(handle, _)| handle).collect()
    }

    fn collect_indices(&self) -> Vec<u32> {
        self.iter().map(|(handle, _)| handle.slot()).collect()
    }

    fn slot_to_index(&self, slot: u32) -> Option<Index> {
        self.contains_slot(slot)
    }
}

// And a couple more custom colors.
// This doesn't follow any standard color naming scheme.
/// A blue you can actually see
pub const BLUE2: Color = Color::new(0.0, 0.2, 1.0, 1.0);
pub const CYAN: Color = Color::new(0.0, 1.0, 1.0, 1.0);

// These intentionally collide with macroquad functions
// which load files from disk or over the network
// to prevent accidentally using them.
// Instead, bundle assets into the binary
// and allow overriding from disk if the file exists.
pub fn load_file() {}
pub fn load_image() {}
pub fn load_sound() {}
pub fn load_string() {}
pub fn load_texture() {}
pub fn load_ttf_font() {}

// For easly switching between f32 and f64.
// Currently only (meant to be) used in debug code.
#[allow(non_camel_case_types)]
pub type fl = f64;

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_v() {
        assert_eq!(v!(-42 420.69), Vec2f::new(-42.0, 420.69));

        // Negative numbers separated by spaces are parsed correctly by rustc
        // but rustfmt formats them as subtraction.
        assert_eq!(v!(-1 - 2), Vec2f::new(-1.0, -2.0));

        struct S {
            x: i32,
        }
        let s = S { x: 42 };
        let a = 420.0;
        let b = 0.69;
        assert_eq!(v!(-s.x, a + b), Vec2f::new(-42.0, 420.69));
    }
}

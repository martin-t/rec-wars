//! A bunch of QoL consts, macros, traits and functions
//! to make gamedev in Rust faster and saner.
//!
//! Should be imported in most files via
//! `use crate::prelude::*`.

// The whole point of this mod is to provide a bunch of stuff
// that may or may not be used but should be *available*.
#![allow(unused_imports)]
#![allow(dead_code)]

// Public re-exports.
// Make the most commonly used types available without importing manually.
// Criteria for inclusion: used often and unlikely to collide.

pub use std::default::Default;

// Intentionally not re-exporting `macroquad::prelude::*` here because:
//  - It contain a lot of stuff not neeced in RecWars.
//  - It causes collisions with rand because it has its own random implementation.
//  - It's only used in a handful of files and the rest of the gamecode doesn't need to know about macroquad.
pub use fnv::{FnvHashMap, FnvHashSet};
pub use rand::prelude::*;
pub use thunderdome::{Arena, Index};
pub use vek::{Clamp, Lerp, Slerp, Wrap};

pub use crate::{
    cvars::{CVec3, Cvars},
    entities::*,
    game_state::*,
    map::{F64Ext, Map, Mat2f, Vec2f, Vec2u, VecExt},
    mq::Assets,
};

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

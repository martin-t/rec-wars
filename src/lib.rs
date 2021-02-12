//! The lib contains code which is shared among the browser and native versions.
//! It also exports structs and entry points for the WASM+canvas browser version
//! (look for items annotated with `wasm_bindgen`).

// Additional warnings that are allow by default (`rustc -W help`)
//#![warn(missing_copy_implementations)] TODO Keep?
#![warn(missing_debug_implementations)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unused)]
#![warn(clippy::all)]
#![allow(clippy::needless_range_loop)] // False positives

#[macro_use]
pub mod debugging; // keep first so the macros are available everywhere

pub mod cvars;
pub mod entities;
pub mod game_state;
pub mod map;
pub mod server;
mod systems;
pub mod timing;

#[cfg(feature = "raw_canvas")]
mod raw_canvas;

const BOT_NAMES: [&str; 20] = [
    "Dr. Dead",
    "Sir Hurt",
    "Mr. Pain",
    "PhD. Torture",
    "Mrs. Chestwound",
    "Ms. Dismember",
    "Don Lobotomy",
    "Lt. Dead",
    "Sgt. Dead",
    "Private Dead",
    "Colonel Dead",
    "Captain Dead",
    "Major Dead",
    "Commander Dead",
    "Díotóir",
    "Fireman",
    "Goldfinger",
    "Silverfinger",
    "Bronzefinger",
    "President Dead",
];

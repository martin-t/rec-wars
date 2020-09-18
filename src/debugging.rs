//! Debugging tools for logging and visualizing what is going on.
//! Use the `dbg_*` macros since they don't need to be imported explicitly
//! and they allow a primitive version of overloading.
//! The fns and structs are only public because the macros need them.

#![allow(unused)]

use std::cell::RefCell;

use crate::map::Vec2f;

pub struct Line {
    pub begin: Vec2f,
    pub end: Vec2f,
    /// Time left (decreases every frame)
    pub time: f64,
    pub color: &'static str,
}

pub struct Cross {
    pub point: Vec2f,
    /// Time left (decreases every frame)
    pub time: f64,
    pub color: &'static str,
}

thread_local! {
    /// Lines of text to be printed onto the screen, cleared after printing.
    pub static DEBUG_TEXTS: RefCell<Vec<String>> = RefCell::new(Vec::new());
    pub static DEBUG_LINES: RefCell<Vec<Line>> = RefCell::new(Vec::new());
    pub static DEBUG_CROSSES: RefCell<Vec<Cross>> = RefCell::new(Vec::new());
}

/// Print text into the console. Uses `println!(..)`-style formatting.
#[macro_export]
macro_rules! dbg_logf {
    ( $( $t:tt )* ) => {
        let s = format!( $( $t )* );
        web_sys::console::log_1(&s.into());
    };
}

/// Print variables into console formatted as `var1: value1, var2: value2`
#[macro_export]
macro_rules! dbg_logd {
    ( $( $e:expr ),* ) => {
        let s = $crate::__print_pairs!( $( $e ),* );
        web_sys::console::log_1(&s.into());
    };
}

/// Print text onto the screen. Uses `println!(..)`-style formatting.
///
/// Useful for printing debug info each frame.
#[macro_export]
macro_rules! dbg_textf {
    ( $( $t:tt )* ) => {
        let s = format!( $( $t )* );
        crate::debugging::DEBUG_TEXTS.with(|texts| {
            texts.borrow_mut().push(s)
        });
    };
}

/// Print variables onto the screen formatted as `var1: value1, var2: value2`
///
/// Useful for printing debug info each frame.
#[macro_export]
macro_rules! dbg_textd {
    ( $( $e:expr ),* ) => {
        let s = $crate::__print_pairs!( $( $e ),* );
        $crate::debugging::DEBUG_TEXTS.with(|texts| {
            texts.borrow_mut().push(s)
        });
    };
}

/// Private helper to print the name and value of each given variable.
/// Not meant to be used directly.
#[macro_export]
macro_rules! __print_pairs {
    ( $e:expr ) => {
        format!("{}: {:.6?}", stringify!($e), $e)
    };
    ( $e:expr, $( $rest:expr ),+ ) => {
        format!(
            "{}, {}",
            $crate::__print_pairs!($e),
            $crate::__print_pairs!( $( $rest ),+ )
        )
    };
}

/// Draw a line between world coordinates.
/// Optionally specify
/// - how long it lasts in seconds (default 1 frame)
/// - color
#[macro_export]
macro_rules! dbg_line {
    ($begin:expr, $end:expr, $time:expr, $color:expr) => {
        $crate::debugging::debug_line($begin, $end, $time, $color);
    };
    ($begin:expr, $end:expr, $time:expr) => {
        $crate::dbg_line!($begin, $end, $time, "red");
    };
    ($begin:expr, $end:expr) => {
        $crate::dbg_line!($begin, $end, 0.0);
    };
}

/// Draw a line between world coordinates.
pub fn debug_line(begin: Vec2f, end: Vec2f, time: f64, color: &'static str) {
    DEBUG_LINES.with(|lines| {
        let line = Line {
            begin,
            end,
            time,
            color,
        };
        lines.borrow_mut().push(line);
    });
}

/// Draw a small cross at world coordinates.
/// Optionally specify
/// - how long it lasts in seconds (default 1 frame)
/// - color
#[macro_export]
macro_rules! dbg_cross {
    ($point:expr, $time:expr, $color:expr) => {
        $crate::debugging::debug_cross($point, $time, $color);
    };
    ($point:expr, $time:expr) => {
        $crate::dbg_cross!($point, $time, "red");
    };
    ($point:expr) => {
        $crate::dbg_cross!($point, 0.0);
    };
}

/// Draw a small cross at world coordinates which lasts `time` seconds.
/// If `time` is 0, it'll last 1 frame.
pub fn debug_cross(point: Vec2f, time: f64, color: &'static str) {
    DEBUG_CROSSES.with(|crosses| {
        let cross = Cross { point, time, color };
        crosses.borrow_mut().push(cross);
    });
}

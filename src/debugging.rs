#![allow(unused)]

use std::cell::RefCell;

use crate::map::Vec2f;

pub struct Line {
    pub begin: Vec2f,
    pub end: Vec2f,
    pub color: &'static str,
}

pub struct Cross {
    pub point: Vec2f,
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

// Macros are more convenient:
//  - no need to rename when adding color
//  - can be used without importing this module

/// Draw line between world coordinates.
#[macro_export]
macro_rules! dbg_line {
    ($begin:expr, $end:expr, $color:expr) => {
        $crate::debugging::debug_line_color($begin, $end, $color);
    };
    ($begin:expr, $end:expr) => {
        $crate::debugging::debug_line($begin, $end);
    };
}

/// Draw line between world coordinates.
pub fn debug_line(begin: Vec2f, end: Vec2f) {
    debug_line_color(begin, end, "red");
}

/// Draw line between world coordinates.
pub fn debug_line_color(begin: Vec2f, end: Vec2f, color: &'static str) {
    DEBUG_LINES.with(|lines| {
        let line = Line { begin, end, color };
        lines.borrow_mut().push(line);
    });
}

/// Draw a small cross at world coordinates.
#[macro_export]
macro_rules! dbg_cross {
    ($point:expr, $color:expr) => {
        $crate::debugging::debug_cross_color($point, $color);
    };
    ($point:expr) => {
        $crate::debugging::debug_cross($point);
    };
}

/// Draw a small cross at world coordinates.
pub fn debug_cross(point: Vec2f) {
    debug_cross_color(point, "red");
}

/// Draw a small cross at world coordinates.
pub fn debug_cross_color(point: Vec2f, color: &'static str) {
    DEBUG_CROSSES.with(|crosses| {
        let cross = Cross { point, color };
        crosses.borrow_mut().push(cross);
    });
}

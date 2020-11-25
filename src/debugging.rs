//! Debugging tools for logging and visualizing what is going on.
//! Use the `dbg_*` macros since they don't need to be imported explicitly
//! and they allow a primitive version of overloading.
//! The fns and structs are only public because the macros need them.

#![allow(dead_code)]

use std::cell::RefCell;

use crate::map::Vec2f;

#[derive(Debug, Clone)]
pub(crate) struct WorldText {
    pub(crate) msg: String,
    pub(crate) pos: Vec2f,
}

#[derive(Debug, Clone)]
pub(crate) struct Line {
    pub(crate) begin: Vec2f,
    pub(crate) end: Vec2f,
    /// Time left (decreases every frame)
    pub(crate) time: f64,
    pub(crate) color: &'static str,
}

#[derive(Debug, Clone)]
pub(crate) struct Cross {
    pub(crate) point: Vec2f,
    /// Time left (decreases every frame)
    pub(crate) time: f64,
    pub(crate) color: &'static str,
}

thread_local! {
    /// Lines of text to be printed onto the screen, cleared after printing.
    pub(crate) static DEBUG_TEXTS: RefCell<Vec<String>> = RefCell::new(Vec::new());
    pub(crate) static DEBUG_TEXTS_WORLD: RefCell<Vec<WorldText>> = RefCell::new(Vec::new());
    pub(crate) static DEBUG_LINES: RefCell<Vec<Line>> = RefCell::new(Vec::new());
    pub(crate) static DEBUG_CROSSES: RefCell<Vec<Cross>> = RefCell::new(Vec::new());
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
        $crate::debugging::DEBUG_TEXTS.with(|texts| {
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
        let s = $crate::__format_pairs!( $( $e ),* );
        $crate::debugging::DEBUG_TEXTS.with(|texts| {
            texts.borrow_mut().push(s)
        });
    };
}

#[macro_export]
macro_rules! dbg_world_textf {
    ( $pos:expr, $( $t:tt )* ) => {
        let s = format!( $( $t )* );
        let text = $crate::debugging::WorldText {
            msg: s,
            pos: $pos,
        };
        $crate::debugging::DEBUG_TEXTS_WORLD.with(|texts| {
            texts.borrow_mut().push(text)
        });
    };
}

/// Private helper to print the name and value of each given variable.
/// Not meant to be used directly.
#[macro_export]
macro_rules! __format_pairs {
    ( $e:expr ) => {
        format!("{}: {:.6?}", stringify!($e), $e)
    };
    ( $e:expr, $( $rest:expr ),+ ) => {
        format!(
            "{}, {}",
            $crate::__format_pairs!($e),
            $crate::__format_pairs!( $( $rest ),+ )
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

/// Helper function, prefer `dbg_line!()` instead.
pub(crate) fn debug_line(begin: Vec2f, end: Vec2f, time: f64, color: &'static str) {
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
/// - how long it lasts in seconds (default is 0.0 which means 1 frame)
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

/// Helper function, prefer `dbg_cross!()` instead.
pub(crate) fn debug_cross(point: Vec2f, time: f64, color: &'static str) {
    DEBUG_CROSSES.with(|crosses| {
        let cross = Cross { point, time, color };
        crosses.borrow_mut().push(cross);
    });
}

/// Count how many times in iterator returned `Some`
/// and print it when it's done.
///
/// # Examples
/// ```ignore
/// query.iter(world).dbg_count("entity count") { ... }
/// ```
pub(crate) trait DbgCount<T>
where
    T: Iterator,
{
    fn dbg_count(self, name: &'static str) -> DbgCounter<T>;
}

impl<T> DbgCount<T> for T
where
    T: Iterator,
{
    fn dbg_count(self, name: &'static str) -> DbgCounter<T> {
        DbgCounter {
            name,
            iterator: self,
            cnt: 0,
        }
    }
}

pub(crate) struct DbgCounter<T>
where
    T: Iterator,
{
    name: &'static str,
    iterator: T,
    cnt: usize,
}

impl<T> Iterator for DbgCounter<T>
where
    T: Iterator,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iterator.next() {
            Some(item) => {
                self.cnt += 1;
                Some(item)
            }
            None => {
                dbg_textf!("{}: {}", self.name, self.cnt);
                None
            }
        }
    }
}

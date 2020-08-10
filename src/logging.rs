use std::cell::RefCell;

/// A macro to provide `println!(..)`-style syntax for `console.log` logging
#[macro_export]
macro_rules! log {
    ( $( $t:tt ),* ) => {
        web_sys::console::log_1(&format!( $( $t ),* ).into());
    };
}

/// Print variables into console formatted as `var1: value1, val2: value2`
#[macro_export]
macro_rules! logd {
    ( $( $t:tt ),* ) => {
        let s = __print_pairs!( $( $t ),* );
        web_sys::console::log_1(&s.into());
    };
}

/// Helper to print the name and value of each given variable
#[macro_export]
macro_rules! __print_pairs {
    ( $t:tt ) => {
        format!("{}: {:.3}", stringify!($t), $t)
    };
    ( $t:tt, $( $rest:tt ),+ ) => {
        format!(
            "{}, {}",
            __print_pairs!($t),
            __print_pairs!( $( $rest ),+ )
        )
    };
}

thread_local!(pub static DEBUG_TEXTS: RefCell<Vec<String>> = RefCell::new(Vec::new()));

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

/// Private helper to print the name and value of each given variable.
/// Not meant to be used directly.
#[macro_export]
macro_rules! __print_pairs {
    ( $e:expr ) => {
        format!("{}: {:.3}", stringify!($e), $e)
    };
    ( $e:expr, $( $rest:expr ),+ ) => {
        format!(
            "{}, {}",
            __print_pairs!($e),
            __print_pairs!( $( $rest ),+ )
        )
    };
}

thread_local!(pub static DEBUG_TEXTS: RefCell<Vec<String>> = RefCell::new(Vec::new()));

#[macro_export]
macro_rules! debug_text {
    ( $( $e:expr ),* ) => {
        let s = __print_pairs!( $( $e ),* );
        crate::logging::DEBUG_TEXTS.with(|texts| {
            texts.borrow_mut().push(s)
        });
    };
}

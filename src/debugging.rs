use std::cell::RefCell;

thread_local! {
    /// Lines of text to be printed onto the screen, cleared after printing.
    pub static DEBUG_TEXTS: RefCell<Vec<String>> = RefCell::new(Vec::new())
}

/// Print text into the console. Uses `println!(..)`-style formatting.
#[macro_export]
macro_rules! logf {
    ( $( $t:tt )* ) => {
        let s = format!( $( $t )* );
        web_sys::console::log_1(&s.into());
    };
}

/// Print variables into console formatted as `var1: value1, var2: value2`
#[macro_export]
macro_rules! logd {
    ( $( $e:expr ),* ) => {
        let s = __print_pairs!( $( $e ),* );
        web_sys::console::log_1(&s.into());
    };
}

/// Print text onto the screen. Uses `println!(..)`-style formatting.
///
/// Useful for printing debug info each frame.
#[macro_export]
macro_rules! dbgf {
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
macro_rules! dbgd {
    ( $( $e:expr ),* ) => {
        let s = __print_pairs!( $( $e ),* );
        crate::debugging::DEBUG_TEXTS.with(|texts| {
            texts.borrow_mut().push(s)
        });
    };
}

/// Private helper to print the name and value of each given variable.
/// Not meant to be used directly.
#[macro_export]
macro_rules! __print_pairs {
    ( $e:expr ) => {
        format!("{}: {:.3?}", stringify!($e), $e)
    };
    ( $e:expr, $( $rest:expr ),+ ) => {
        format!(
            "{}, {}",
            __print_pairs!($e),
            __print_pairs!( $( $rest ),+ )
        )
    };
}

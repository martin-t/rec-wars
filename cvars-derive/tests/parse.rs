use cvars_derive::Cvars;

#[derive(Debug, Clone, Cvars)]
struct Cvars {
    pub g_bool: bool,
    pub g_int: i32,
    pub g_usize: usize,
    pub g_float: f32,
    pub g_double: f64,
    pub g_enum: Enum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enum {
    One,
    Two,
}

fn main() {}

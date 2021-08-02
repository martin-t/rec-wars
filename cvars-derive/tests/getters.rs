use cvars_derive::Cvars;

#[derive(Debug, Clone, Cvars)]
struct Cvars {
    // TODO parse all
    //pub g_bool: bool,
    pub g_int: i32,
    //pub g_usize: usize,
    //pub g_float: f32,
    //pub g_double: f64,
    //pub g_enum: Enum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enum {
    One,
    Two,
}

fn main() {
    let cvars = Cvars { g_int: 42 };
    assert_eq!(cvars.get("g_int"), cvars.g_int);
}

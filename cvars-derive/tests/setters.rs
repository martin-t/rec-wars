use cvars_derive::Cvars;

#[derive(Debug, Clone, Default, Cvars)]
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

impl Default for Enum {
    fn default() -> Self {
        Enum::One
    }
}

fn main() {
    let mut cvars = Cvars::default();

    cvars.set("g_bool", true);
    cvars.set("g_int", 42);
    cvars.set("g_usize", 987654_usize);
    cvars.set("g_float", 5.0_f32);
    cvars.set("g_double", 10.0);
    cvars.set("g_enum", Enum::Two);

    assert_eq!(cvars.g_bool, true);
    assert_eq!(cvars.g_int, 42);
    assert_eq!(cvars.g_usize, 987654);
    assert_eq!(cvars.g_float, 5.0);
    assert_eq!(cvars.g_double, 10.0);
    assert_eq!(cvars.g_enum, Enum::Two);
}

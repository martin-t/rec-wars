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

fn main() {
    let cvars = Cvars {
        g_bool: true,
        g_int: 42,
        g_usize: 987654,
        g_float: 5.0,
        g_double: 10.0,
        g_enum: Enum::Two,
    };
    // Creating a second struct so that type inferrence works.
    // Just `assert_eq!(cvars.get("g_int"), cvars.g_int);`
    // would require specifying types, same for using `==`.
    let other = Cvars {
        g_bool: cvars.get("g_bool"),
        g_int: cvars.get("g_int"),
        g_usize: cvars.get("g_usize"),
        g_float: cvars.get("g_float"),
        g_double: cvars.get("g_double"),
        g_enum: cvars.get("g_enum"),
    };
    assert_eq!(other.g_bool, cvars.g_bool);
    assert_eq!(other.g_int, cvars.g_int);
    assert_eq!(other.g_usize, cvars.g_usize);
    assert_eq!(other.g_float, cvars.g_float);
    assert_eq!(other.g_double, cvars.g_double);
    assert_eq!(other.g_enum, cvars.g_enum);
}

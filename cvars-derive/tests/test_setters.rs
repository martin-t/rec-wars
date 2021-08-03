mod shared;

use shared::{Cvars, Enum};

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

mod shared;

use shared::{Cvars, Enum};

fn main() {
    let mut cvars = Cvars::default();

    cvars.set_str("g_bool", "true");
    cvars.set_str("g_int", "42");
    cvars.set_str("g_usize", "987654");
    cvars.set_str("g_float", "5.0");
    cvars.set_str("g_double", "10.0");
    cvars.set_str("g_enum", "two");

    assert_eq!(cvars.g_bool, true);
    assert_eq!(cvars.g_int, 42);
    assert_eq!(cvars.g_usize, 987654);
    assert_eq!(cvars.g_float, 5.0);
    assert_eq!(cvars.g_double, 10.0);
    assert_eq!(cvars.g_enum, Enum::Two);
}

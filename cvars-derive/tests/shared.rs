use std::str::FromStr;

use cvars_derive::Cvars;

#[derive(Debug, Clone, Default, Cvars)]
pub struct Cvars {
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

impl FromStr for Enum {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "one" => Ok(Enum::One),
            "two" => Ok(Enum::Two),
            _ => Err(format!("Couldn't parse {} as Enum", s)),
        }
    }
}

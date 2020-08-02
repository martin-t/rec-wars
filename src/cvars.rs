use std::default::Default;

use wasm_bindgen::prelude::*;

/// Prefix meanings:
/// g_ is gameplay
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Cvars {
    pub g_guided_missile_speed: f64,
    pub g_guided_missile_turn_rate_increase: f64,
    pub g_guided_missile_turn_rate_decrease: f64,
    pub g_guided_missile_turn_rate_max: f64,
}

#[wasm_bindgen]
impl Cvars {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Cvars {
    fn default() -> Self {
        Self {
            g_guided_missile_speed: 50.0,
            g_guided_missile_turn_rate_increase: 2.5,
            g_guided_missile_turn_rate_decrease: 0.3,
            g_guided_missile_turn_rate_max: 2.5,
        }
    }
}

use std::default::Default;

use wasm_bindgen::prelude::*;

/// Prefix meanings:
/// g_ is gameplay
/// r_ is rendering
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Cvars {
    // Would be nice to keep alphabetically.
    // Long-term this needs some kind of better system to reduce duplication / manual work.
    pub g_guided_missile_speed_change: f64,
    pub g_guided_missile_speed_initial: f64,
    pub g_guided_missile_speed_max: f64,
    pub g_guided_missile_speed_min: f64,
    pub g_guided_missile_turn_rate_increase: f64,
    pub g_guided_missile_turn_rate_decrease: f64,
    pub g_guided_missile_turn_rate_max: f64,
    pub r_align_to_pixels_background: bool,
    pub r_smoothing: bool,
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
            // TODO this is all per-frame for now
            g_guided_missile_speed_change: 0.01,
            g_guided_missile_speed_initial: 0.36,
            g_guided_missile_speed_max: 0.5,
            g_guided_missile_speed_min: 0.3,
            g_guided_missile_turn_rate_increase: 0.0035,
            g_guided_missile_turn_rate_decrease: 0.0025,
            g_guided_missile_turn_rate_max: 0.0525,
            r_align_to_pixels_background: true,
            r_smoothing: false,
        }
    }
}

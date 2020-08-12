use std::default::Default;

use wasm_bindgen::prelude::*;

/// Prefix meanings:
/// d_ is debug
/// g_ is gameplay
/// r_ is rendering
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Cvars {
    // Would be nice to keep alphabetically.
    // Long-term this needs some kind of better system to reduce duplication / manual work.
    /// Change speed of everything in the game
    pub d_speed: f64,

    pub g_guided_missile_speed_change: f64,
    pub g_guided_missile_speed_initial: f64,
    pub g_guided_missile_speed_max: f64,
    pub g_guided_missile_speed_min: f64,
    pub g_guided_missile_turn_rate_increase: f64,
    /// Fraction left after 1 s. At first decreases fast, then slower.
    pub g_guided_missile_turn_rate_friction: f64,
    /// Linear decrease to stop completely
    pub g_guided_missile_turn_rate_decrease: f64,
    pub g_guided_missile_turn_rate_max: f64,

    pub g_tank_accel_backward: f64,
    pub g_tank_accel_forward: f64,
    pub g_tank_friction: f64,
    pub g_tank_turn_rate_friction_linear: f64,
    pub g_tank_turn_rate_friction_const: f64,
    pub g_tank_turn_rate_increase: f64,

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
            d_speed: 1.0,

            g_guided_missile_speed_change: 600.0,
            g_guided_missile_speed_initial: 360.0,
            g_guided_missile_speed_max: 500.0,
            g_guided_missile_speed_min: 300.0,
            g_guided_missile_turn_rate_increase: 12.6,
            g_guided_missile_turn_rate_friction: 0.01,
            g_guided_missile_turn_rate_decrease: 1.0,
            g_guided_missile_turn_rate_max: 3.15,

            g_tank_accel_backward: 500.0,
            g_tank_accel_forward: 500.0,
            g_tank_friction: 0.1,
            g_tank_turn_rate_friction_linear: 0.04,
            g_tank_turn_rate_friction_const: 0.02,
            g_tank_turn_rate_increase: 0.2,

            r_align_to_pixels_background: true,
            r_smoothing: false,
        }
    }
}

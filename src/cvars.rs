//! Console variables - configuration options for anything and everything.

use std::default::Default;

use wasm_bindgen::prelude::*;

/// Console variables - configuration options for anything and everything.
///
/// Prefix meanings:
/// d_ is debug
/// g_ is gameplay
/// hud_ is the heads-up display
/// r_ is rendering
/// sv_ is server administration + performance
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Cvars {
    // Would be nice to keep alphabetically.
    // Long-term this needs some kind of better system to reduce duplication / manual work.
    //  |
    //  v
    pub d_debug_draw: bool,
    pub d_debug_text: bool,
    pub d_debug_text_line_height: f64,
    pub d_rockets_image: bool,
    pub d_seed: u64,
    /// Change speed of everything in the game
    pub d_speed: f64,

    pub g_explosion_duration: f64,

    pub g_machine_gun_add_vehicle_velocity: bool,
    pub g_machine_gun_speed: f64,
    pub g_machine_gun_trail_length: f64,

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

    pub g_rockets_add_vehicle_velocity: bool,
    pub g_rockets_explosion_size: f64,
    pub g_rockets_speed: f64,

    pub g_tank_accel_backward: f64,
    pub g_tank_accel_forward: f64,
    pub g_tank_friction_const: f64,
    pub g_tank_friction_linear: f64,
    pub g_tank_maxs_x: f64,
    pub g_tank_maxs_y: f64,
    pub g_tank_mins_x: f64,
    pub g_tank_mins_y: f64,
    pub g_tank_turn_effectiveness: f64,
    pub g_tank_turn_rate_friction_const: f64,
    pub g_tank_turn_rate_friction_linear: f64,
    pub g_tank_turn_rate_increase: f64,
    pub g_tank_turn_rate_max: f64,
    pub g_tank_speed_max: f64,

    pub hud_charge_x: f64,
    pub hud_charge_y: f64,
    /// Original RecWar had 99.
    pub hud_charge_width: f64,
    /// Original RecWar had 4.
    pub hud_charge_height: f64,

    pub hud_hp_x: f64,
    pub hud_hp_y: f64,
    /// Original RecWar had 99.
    pub hud_hp_width: f64,
    /// Original RecWar had 9.
    pub hud_hp_height: f64,

    pub hud_missile_indicator_dash_length: f64,
    pub hud_missile_indicator_radius: f64,

    pub hud_weapon_icon_shadow_alpha: f64,
    pub hud_weapon_icon_shadow_x: f64,
    pub hud_weapon_icon_shadow_y: f64,
    pub hud_weapon_icon_x: f64,
    pub hud_weapon_icon_y: f64,

    pub r_align_to_pixels_background: bool,
    pub r_smoothing: bool,

    pub sv_gamelogic_mode: TickrateMode,
    pub sv_gamelogic_fixed_fps: f64,
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
            d_debug_draw: true,
            d_debug_text: true,
            d_debug_text_line_height: 11.0,
            d_rockets_image: true,
            d_seed: 6,
            d_speed: 1.0,

            g_explosion_duration: 0.5,

            g_machine_gun_add_vehicle_velocity: true,
            g_machine_gun_speed: 1000.0,
            g_machine_gun_trail_length: 8.0,

            g_guided_missile_speed_change: 600.0,
            g_guided_missile_speed_initial: 360.0,
            g_guided_missile_speed_max: 500.0,
            g_guided_missile_speed_min: 300.0,
            g_guided_missile_turn_rate_increase: 12.6,
            g_guided_missile_turn_rate_friction: 0.99,
            g_guided_missile_turn_rate_decrease: 1.0,
            g_guided_missile_turn_rate_max: 3.15,
            g_rockets_add_vehicle_velocity: true,
            g_rockets_explosion_size: 0.5,
            g_rockets_speed: 600.0,

            g_tank_accel_backward: 550.0,
            g_tank_accel_forward: 550.0,
            g_tank_friction_const: 50.0,
            g_tank_friction_linear: 0.9,
            g_tank_maxs_x: 19.0,
            g_tank_maxs_y: 12.0,
            g_tank_mins_x: -19.0,
            g_tank_mins_y: -12.0,
            g_tank_turn_effectiveness: 0.5,
            g_tank_turn_rate_friction_const: 0.05,
            g_tank_turn_rate_friction_linear: 0.96,
            g_tank_turn_rate_increase: 0.2,
            g_tank_turn_rate_max: f64::INFINITY,
            g_tank_speed_max: 250.0,

            hud_charge_x: 30.0,
            hud_charge_y: 770.0,
            hud_charge_width: 100.0,
            hud_charge_height: 4.0,

            hud_hp_x: 30.0,
            hud_hp_y: 750.0,
            hud_hp_width: 100.0,
            hud_hp_height: 9.0,

            hud_missile_indicator_dash_length: 3.3,
            hud_missile_indicator_radius: 18.0,

            hud_weapon_icon_shadow_alpha: 0.5,
            hud_weapon_icon_shadow_x: 2.0,
            hud_weapon_icon_shadow_y: 2.0,
            hud_weapon_icon_x: 170.0,
            hud_weapon_icon_y: 772.0,

            r_align_to_pixels_background: true,
            r_smoothing: false,

            sv_gamelogic_mode: TickrateMode::Synchronized,
            sv_gamelogic_fixed_fps: 150.0,
        }
    }
}

/// Various options how to handle different physics/gamelogic and rendering framerates.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickrateMode {
    /// Same FPS as rendering - runs one tick with variable timestep before rendering.
    /// This means simulation always catches up to rendering (wall-clock time) exactly.
    Synchronized,
    /// Same FPS as rendering unless the step would be too large or too small.
    /// Too large steps are split into smaller ones.
    /// Too small steps are skipped and the time carries over to the next render frame,
    /// this means simulation can be slightly behind what should be rendered.
    SynchronizedBounded,
    /// Fixed FPS - always the same timestep, leftover time carries over to the next render frame.
    /// This means simulation can be slightly behind what should be rendered.
    Fixed,
    /// Simulation runs in fixed steps as long as it can, the last step is smaller
    /// to catch up to rendering exactly. The last step is then thrown away and simulation
    /// resumes from the last full step so it's deterministic.
    /// TODO what with too small steps?
    /// TODO link to john blow vid
    FixedOrSmaller,
    /// TODO doc
    FixedWithInterpolation,
}

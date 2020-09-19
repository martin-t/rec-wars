//! Console variables - configuration options for anything and everything.

use std::default::Default;

use wasm_bindgen::prelude::*;

use crate::{components::Vehicle, map::Vec2f, weapons::*};

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
    // Long-term this needs some kind of better system to reduce duplication / manual work.
    // Veloren uses https://crates.io/crates/const-tweaker
    // Would be nice to keep alphabetically.
    //  |
    //  v
    pub d_debug_draw: bool,
    pub d_debug_text: bool,
    pub d_debug_text_line_height: f64,
    pub d_rockets_image: bool,
    pub d_seed: u64,
    /// Change speed of everything in the game
    pub d_speed: f64,

    pub g_bfg_add_vehicle_velocity: bool,
    pub g_bfg_radius: f64,
    pub g_bfg_reload_ammo: u32,
    pub g_bfg_reload_time: f64,
    pub g_bfg_speed: f64,

    pub g_cluster_bomb_add_vehicle_velocity: bool,
    pub g_cluster_bomb_count: i32,
    pub g_cluster_bomb_explosion_scale: f64,
    pub g_cluster_bomb_reload_ammo: u32,
    pub g_cluster_bomb_reload_time: f64,
    pub g_cluster_bomb_shadow_alpha: f64,
    pub g_cluster_bomb_shadow_x: f64,
    pub g_cluster_bomb_shadow_y: f64,
    pub g_cluster_bomb_size: f64,
    pub g_cluster_bomb_speed: f64,
    pub g_cluster_bomb_speed_spread_forward: f64,
    pub g_cluster_bomb_speed_spread_gaussian: bool,
    pub g_cluster_bomb_speed_spread_sideways: f64,
    pub g_cluster_bomb_time: f64,
    pub g_cluster_bomb_time_spread: f64,

    pub g_homing_missile_reload_ammo: u32,
    pub g_homing_missile_reload_time: f64,

    pub g_machine_gun_add_vehicle_velocity: bool,
    pub g_machine_gun_angle_spread: f64,
    pub g_machine_gun_refire: f64,
    pub g_machine_gun_reload_ammo: u32,
    pub g_machine_gun_reload_time: f64,
    pub g_machine_gun_speed: f64,
    pub g_machine_gun_trail_length: f64,

    pub g_guided_missile_reload_ammo: u32,
    pub g_guided_missile_reload_time: f64,
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

    pub g_hardpoint_tank_machine_gun: Hardpoint,
    pub g_hardpoint_tank_machine_gun_x: f64,
    pub g_hardpoint_tank_machine_gun_y: f64,
    pub g_hardpoint_tank_railgun: Hardpoint,
    pub g_hardpoint_tank_railgun_x: f64,
    pub g_hardpoint_tank_railgun_y: f64,
    pub g_hardpoint_tank_cluster_bomb: Hardpoint,
    pub g_hardpoint_tank_cluster_bomb_x: f64,
    pub g_hardpoint_tank_cluster_bomb_y: f64,
    pub g_hardpoint_tank_rockets: Hardpoint,
    pub g_hardpoint_tank_rockets_x: f64,
    pub g_hardpoint_tank_rockets_y: f64,
    pub g_hardpoint_tank_homing_missile: Hardpoint,
    pub g_hardpoint_tank_homing_missile_x: f64,
    pub g_hardpoint_tank_homing_missile_y: f64,
    pub g_hardpoint_tank_guided_missile: Hardpoint,
    pub g_hardpoint_tank_guided_missile_x: f64,
    pub g_hardpoint_tank_guided_missile_y: f64,
    pub g_hardpoint_tank_bfg: Hardpoint,
    pub g_hardpoint_tank_bfg_x: f64,
    pub g_hardpoint_tank_bfg_y: f64,

    pub g_railgun_reload_ammo: u32,
    pub g_railgun_reload_time: f64,

    pub g_rockets_add_vehicle_velocity: bool,
    pub g_rockets_explosion_scale: f64,
    pub g_rockets_refire: f64,
    pub g_rockets_reload_ammo: u32,
    pub g_rockets_reload_time: f64,
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
    pub g_tank_turret_offset_chassis_x: f64,
    pub g_tank_turret_offset_chassis_y: f64,
    pub g_tank_turret_offset_turret_x: f64,
    pub g_tank_turret_offset_turret_y: f64,
    pub g_tank_speed_max: f64,

    pub g_turret_turn_speed: f64,

    pub hud_ammo_x: f64,
    pub hud_ammo_y: f64,
    /// Original RecWar had 99.
    pub hud_ammo_width: f64,
    /// Original RecWar had 4.
    pub hud_ammo_height: f64,

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
    pub r_draw_cluster_bombs: bool,
    pub r_explosion_duration: f64,
    pub r_explosions_reverse: bool,
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

    pub(crate) fn g_hardpoint(&self, vehicle: Vehicle, weapon: Weapon) -> (Hardpoint, Vec2f) {
        match vehicle {
            Vehicle::Tank => match weapon {
                Weapon::Mg => (
                    self.g_hardpoint_tank_machine_gun,
                    Vec2f::new(
                        self.g_hardpoint_tank_machine_gun_x,
                        self.g_hardpoint_tank_machine_gun_y,
                    ),
                ),
                Weapon::Rail => (
                    self.g_hardpoint_tank_railgun,
                    Vec2f::new(
                        self.g_hardpoint_tank_railgun_x,
                        self.g_hardpoint_tank_railgun_y,
                    ),
                ),
                Weapon::Cb => (
                    self.g_hardpoint_tank_cluster_bomb,
                    Vec2f::new(
                        self.g_hardpoint_tank_cluster_bomb_x,
                        self.g_hardpoint_tank_cluster_bomb_y,
                    ),
                ),
                Weapon::Rockets => (
                    self.g_hardpoint_tank_rockets,
                    Vec2f::new(
                        self.g_hardpoint_tank_rockets_x,
                        self.g_hardpoint_tank_rockets_y,
                    ),
                ),
                Weapon::Hm => (
                    self.g_hardpoint_tank_homing_missile,
                    Vec2f::new(
                        self.g_hardpoint_tank_homing_missile_x,
                        self.g_hardpoint_tank_homing_missile_y,
                    ),
                ),
                Weapon::Gm => (
                    self.g_hardpoint_tank_guided_missile,
                    Vec2f::new(
                        self.g_hardpoint_tank_guided_missile_x,
                        self.g_hardpoint_tank_guided_missile_y,
                    ),
                ),
                Weapon::Bfg => (
                    self.g_hardpoint_tank_bfg,
                    Vec2f::new(self.g_hardpoint_tank_bfg_x, self.g_hardpoint_tank_bfg_y),
                ),
            },
            _ => unimplemented!(),
        }
    }

    pub(crate) fn g_vehicle_turret_offset_chassis(&self, vehicle: Vehicle) -> Vec2f {
        match vehicle {
            Vehicle::Tank => Vec2f::new(
                self.g_tank_turret_offset_chassis_x,
                self.g_tank_turret_offset_chassis_y,
            ),
            _ => unimplemented!(),
        }
    }

    pub(crate) fn g_vehicle_turret_offset_turret(&self, vehicle: Vehicle) -> Vec2f {
        match vehicle {
            Vehicle::Tank => Vec2f::new(
                self.g_tank_turret_offset_turret_x,
                self.g_tank_turret_offset_turret_y,
            ),
            _ => unimplemented!(),
        }
    }

    pub(crate) fn g_weapon_refire(&self, weapon: Weapon) -> f64 {
        match weapon {
            Weapon::Mg => self.g_machine_gun_refire,
            Weapon::Rail => 0.0,
            Weapon::Cb => 0.0,
            Weapon::Rockets => self.g_rockets_refire,
            Weapon::Hm => 0.0,
            Weapon::Gm => 0.0,
            Weapon::Bfg => 0.0,
        }
    }

    pub(crate) fn g_weapon_reload_ammo(&self, weapon: Weapon) -> u32 {
        match weapon {
            Weapon::Mg => self.g_machine_gun_reload_ammo,
            Weapon::Rail => self.g_railgun_reload_ammo,
            Weapon::Cb => self.g_cluster_bomb_reload_ammo,
            Weapon::Rockets => self.g_rockets_reload_ammo,
            Weapon::Hm => self.g_homing_missile_reload_ammo,
            Weapon::Gm => self.g_guided_missile_reload_ammo,
            Weapon::Bfg => self.g_bfg_reload_ammo,
        }
    }

    pub(crate) fn g_weapon_reload_time(&self, weapon: Weapon) -> f64 {
        match weapon {
            Weapon::Mg => self.g_machine_gun_reload_time,
            Weapon::Rail => self.g_railgun_reload_time,
            Weapon::Cb => self.g_cluster_bomb_reload_time,
            Weapon::Rockets => self.g_rockets_reload_time,
            Weapon::Hm => self.g_homing_missile_reload_time,
            Weapon::Gm => self.g_guided_missile_reload_time,
            Weapon::Bfg => self.g_bfg_reload_time,
        }
    }
}

impl Default for Cvars {
    fn default() -> Self {
        Self {
            d_debug_draw: true,
            d_debug_text: true,
            d_debug_text_line_height: 11.0,
            d_rockets_image: true,
            d_seed: 0,
            d_speed: 1.0,

            g_bfg_add_vehicle_velocity: true,
            g_bfg_radius: 4.0,
            g_bfg_reload_ammo: 1,
            g_bfg_reload_time: 2.5,
            g_bfg_speed: 100.0,

            g_cluster_bomb_add_vehicle_velocity: true,
            g_cluster_bomb_count: 40,
            g_cluster_bomb_explosion_scale: 0.5,
            g_cluster_bomb_reload_ammo: 1,
            g_cluster_bomb_reload_time: 1.5,
            g_cluster_bomb_shadow_alpha: 1.0,
            g_cluster_bomb_shadow_x: 2.0,
            g_cluster_bomb_size: 1.2,
            g_cluster_bomb_shadow_y: 2.0,
            g_cluster_bomb_speed: 400.0,
            g_cluster_bomb_speed_spread_forward: 50.0,
            g_cluster_bomb_speed_spread_gaussian: true,
            g_cluster_bomb_speed_spread_sideways: 50.0,
            g_cluster_bomb_time: 0.8,
            g_cluster_bomb_time_spread: 0.2,

            g_homing_missile_reload_ammo: 1,
            g_homing_missile_reload_time: 1.5,

            g_machine_gun_add_vehicle_velocity: true,
            g_machine_gun_angle_spread: 0.015,
            g_machine_gun_refire: 0.050,
            g_machine_gun_reload_ammo: 50,
            g_machine_gun_reload_time: 1.0,
            g_machine_gun_speed: 1000.0,
            g_machine_gun_trail_length: 10.0,

            g_guided_missile_reload_ammo: 1,
            g_guided_missile_reload_time: 1.5,
            g_guided_missile_speed_change: 600.0,
            g_guided_missile_speed_initial: 360.0,
            g_guided_missile_speed_max: 500.0,
            g_guided_missile_speed_min: 300.0,
            g_guided_missile_turn_rate_increase: 12.6,
            g_guided_missile_turn_rate_friction: 0.99,
            g_guided_missile_turn_rate_decrease: 1.0,
            g_guided_missile_turn_rate_max: 3.15,

            g_hardpoint_tank_machine_gun: Hardpoint::Turret,
            g_hardpoint_tank_machine_gun_x: 12.0,
            g_hardpoint_tank_machine_gun_y: -5.0,
            g_hardpoint_tank_railgun: Hardpoint::Turret,
            g_hardpoint_tank_railgun_x: 35.0,
            g_hardpoint_tank_railgun_y: 0.0,
            g_hardpoint_tank_cluster_bomb: Hardpoint::Turret,
            g_hardpoint_tank_cluster_bomb_x: 35.0,
            g_hardpoint_tank_cluster_bomb_y: 0.0,
            g_hardpoint_tank_rockets: Hardpoint::Turret,
            g_hardpoint_tank_rockets_x: 35.0,
            g_hardpoint_tank_rockets_y: 0.0,
            g_hardpoint_tank_homing_missile: Hardpoint::Chassis,
            g_hardpoint_tank_homing_missile_x: 0.0,
            g_hardpoint_tank_homing_missile_y: -15.0,
            g_hardpoint_tank_guided_missile: Hardpoint::Chassis,
            g_hardpoint_tank_guided_missile_x: 0.0,
            g_hardpoint_tank_guided_missile_y: -15.0,
            g_hardpoint_tank_bfg: Hardpoint::Turret,
            g_hardpoint_tank_bfg_x: 35.0,
            g_hardpoint_tank_bfg_y: 0.0,

            g_railgun_reload_ammo: 1,
            g_railgun_reload_time: 1.0,

            g_rockets_add_vehicle_velocity: true,
            g_rockets_explosion_scale: 0.5,
            g_rockets_refire: 0.200,
            g_rockets_reload_ammo: 6,
            g_rockets_reload_time: 1.5,
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
            g_tank_turret_offset_chassis_x: -5.0,
            g_tank_turret_offset_chassis_y: 0.0,
            g_tank_turret_offset_turret_x: -14.0,
            g_tank_turret_offset_turret_y: 0.0,
            g_tank_speed_max: 250.0,

            g_turret_turn_speed: 2.0,

            hud_ammo_x: 30.0,
            hud_ammo_y: 770.0,
            hud_ammo_width: 100.0,
            hud_ammo_height: 4.0,

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
            r_draw_cluster_bombs: true,
            r_explosion_duration: 0.5,
            // After trying true for a while, i think false looks better:
            // - CB looks smoother. With true it sometimes looked like it had 2 stages
            //   because the later explosions were suddenly revealed after the first ones disappeared.
            // - Rockets look better if hitting the same spot.
            r_explosions_reverse: false,
            r_smoothing: false,

            sv_gamelogic_mode: TickrateMode::Synchronized,
            sv_gamelogic_fixed_fps: 150.0,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hardpoint {
    Chassis,
    Turret,
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
    /// This means simulation can be up to almost a frame behind what should be rendered.
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

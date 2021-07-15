//! Console variables - configuration options for anything and everything.

use std::default::Default;

#[cfg(feature = "raw_canvas")]
use wasm_bindgen::prelude::*;

use crate::{entities::Hitbox, entities::VehicleType, entities::Weapon, map::Vec2f};

/// Console variables - configuration options for anything and everything.
///
/// Prefix meanings:
/// d_ is debug
/// g_ is gameplay
/// hud_ is the heads-up display
/// r_ is rendering
/// sv_ is server administration + performance
///
/// This struct is shared with the JS side so it has some limitations:
/// - fields have to be Copy
/// - only C-like enums
/// - avoid fields that are structs
///     - they compile but can't be changed from JS (the change is thrown away)
///     - e.g. `cvars.g_tank.speed` wouldn't work
#[cfg_attr(feature = "raw_canvas", wasm_bindgen)]
#[derive(Debug, Clone)]
#[allow(missing_copy_implementations)]
pub struct Cvars {
    // Long-term this needs some kind of better system to reduce duplication / manual work.
    // Would be nice to keep alphabetically.
    //  |
    //  v
    /// Master switch for AI - disable if you want stationary targets
    pub ai: bool,

    pub bots_max: usize,

    /// "Temporary" cvar for quick testing. Normally unused but kept here
    /// so I don't have to add a cvar each time I want a quick toggle.
    pub d_dbg: bool,

    /// Master switch for debug output - the d_draw_* group.
    pub d_draw: bool,
    pub d_draw_crosses: bool,
    pub d_draw_hitboxes: bool,
    pub d_draw_lines: bool,
    pub d_draw_lines_ends_length: f64,
    pub d_draw_perf: bool,
    pub d_draw_text: bool,
    pub d_draw_text_line_height: f64,
    pub d_draw_world_text: bool,
    pub d_explosion_radius: bool,
    /// Draw FPS counter. Intentionally not in the d_draw_* group
    /// so I can easily check perf with and without the other debug output.
    pub d_fps: bool,
    pub d_fps_period: f64,
    pub d_fps_x: f64,
    pub d_fps_y: f64,
    pub d_timing_samples: usize,
    pub d_tracing: bool,
    /// The seed to initialize the RNG.
    ///
    /// This is not very helpful by itself because by the time you can change cvars in the console,
    /// the seed has already been used.
    /// However, you can set it by appending it to the URL (e.g. `?d_seed=5`) like any other cvar.
    pub d_seed: u64,
    /// Change speed of everything in the game
    pub d_speed: f64,

    /// Scale hit points of all vehicles
    pub g_armor: f64,

    pub g_bfg_beam_damage_per_sec: f64,
    pub g_bfg_beam_range: f64,
    pub g_bfg_damage_direct: f64,
    pub g_bfg_explosion_damage: f64,
    pub g_bfg_explosion_radius: f64,
    pub g_bfg_explosion_scale: f64,
    pub g_bfg_radius: f64,
    pub g_bfg_reload_ammo: u32,
    pub g_bfg_reload_time: f64,
    pub g_bfg_speed: f64,
    pub g_bfg_vehicle_velocity_factor: f64,

    pub g_cluster_bomb_count: i32,
    pub g_cluster_bomb_damage_direct: f64,
    pub g_cluster_bomb_explosion_damage: f64,
    pub g_cluster_bomb_explosion_radius: f64,
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
    pub g_cluster_bomb_vehicle_velocity_factor: f64,

    pub g_ffa_score_kill: i32,
    pub g_ffa_score_death: i32,

    pub g_homing_missile_damage_direct: f64,
    pub g_homing_missile_explosion_damage: f64,
    pub g_homing_missile_explosion_radius: f64,
    pub g_homing_missile_explosion_scale: f64,
    pub g_homing_missile_reload_ammo: u32,
    pub g_homing_missile_reload_time: f64,
    pub g_homing_missile_speed_initial: f64,
    pub g_homing_missile_vehicle_velocity_factor: f64,

    pub g_machine_gun_angle_spread: f64,
    pub g_machine_gun_damage: f64,
    pub g_machine_gun_refire: f64,
    pub g_machine_gun_reload_ammo: u32,
    pub g_machine_gun_reload_time: f64,
    pub g_machine_gun_speed: f64,
    pub g_machine_gun_trail_length: f64,
    pub g_machine_gun_vehicle_velocity_factor: f64,

    pub g_guided_missile_accel_forward: f64,
    pub g_guided_missile_damage_direct: f64,
    pub g_guided_missile_explosion_damage: f64,
    pub g_guided_missile_explosion_radius: f64,
    pub g_guided_missile_explosion_scale: f64,
    pub g_guided_missile_friction_const: f64,
    pub g_guided_missile_friction_linear: f64,
    pub g_guided_missile_reload_ammo: u32,
    pub g_guided_missile_reload_time: f64,
    pub g_guided_missile_speed_initial: f64,
    pub g_guided_missile_speed_max: f64,
    pub g_guided_missile_turn_effectiveness: f64,
    pub g_guided_missile_turn_rate_increase: f64,
    pub g_guided_missile_turn_rate_friction_const: f64,
    pub g_guided_missile_turn_rate_friction_linear: f64,
    pub g_guided_missile_turn_rate_max: f64,
    pub g_guided_missile_vehicle_velocity_factor: f64,

    pub g_hardpoint_hovercraft_machine_gun: Hardpoint,
    pub g_hardpoint_hovercraft_machine_gun_x: f64,
    pub g_hardpoint_hovercraft_machine_gun_y: f64,
    pub g_hardpoint_hovercraft_railgun: Hardpoint,
    pub g_hardpoint_hovercraft_railgun_x: f64,
    pub g_hardpoint_hovercraft_railgun_y: f64,
    pub g_hardpoint_hovercraft_cluster_bomb: Hardpoint,
    pub g_hardpoint_hovercraft_cluster_bomb_x: f64,
    pub g_hardpoint_hovercraft_cluster_bomb_y: f64,
    pub g_hardpoint_hovercraft_rockets: Hardpoint,
    pub g_hardpoint_hovercraft_rockets_x: f64,
    pub g_hardpoint_hovercraft_rockets_y: f64,
    pub g_hardpoint_hovercraft_homing_missile: Hardpoint,
    pub g_hardpoint_hovercraft_homing_missile_x: f64,
    pub g_hardpoint_hovercraft_homing_missile_y: f64,
    pub g_hardpoint_hovercraft_guided_missile: Hardpoint,
    pub g_hardpoint_hovercraft_guided_missile_x: f64,
    pub g_hardpoint_hovercraft_guided_missile_y: f64,
    pub g_hardpoint_hovercraft_bfg: Hardpoint,
    pub g_hardpoint_hovercraft_bfg_x: f64,
    pub g_hardpoint_hovercraft_bfg_y: f64,

    pub g_hardpoint_hummer_machine_gun: Hardpoint,
    pub g_hardpoint_hummer_machine_gun_x: f64,
    pub g_hardpoint_hummer_machine_gun_y: f64,
    pub g_hardpoint_hummer_railgun: Hardpoint,
    pub g_hardpoint_hummer_railgun_x: f64,
    pub g_hardpoint_hummer_railgun_y: f64,
    pub g_hardpoint_hummer_cluster_bomb: Hardpoint,
    pub g_hardpoint_hummer_cluster_bomb_x: f64,
    pub g_hardpoint_hummer_cluster_bomb_y: f64,
    pub g_hardpoint_hummer_rockets: Hardpoint,
    pub g_hardpoint_hummer_rockets_x: f64,
    pub g_hardpoint_hummer_rockets_y: f64,
    pub g_hardpoint_hummer_homing_missile: Hardpoint,
    pub g_hardpoint_hummer_homing_missile_x: f64,
    pub g_hardpoint_hummer_homing_missile_y: f64,
    pub g_hardpoint_hummer_guided_missile: Hardpoint,
    pub g_hardpoint_hummer_guided_missile_x: f64,
    pub g_hardpoint_hummer_guided_missile_y: f64,
    pub g_hardpoint_hummer_bfg: Hardpoint,
    pub g_hardpoint_hummer_bfg_x: f64,
    pub g_hardpoint_hummer_bfg_y: f64,

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

    pub g_hitcircle_radius: f64, // TODO proper hitbox

    pub g_hovercraft_accel_backward: f64,
    pub g_hovercraft_accel_forward: f64,
    pub g_hovercraft_friction_const: f64,
    pub g_hovercraft_friction_linear: f64,
    pub g_hovercraft_base_hp: f64,
    pub g_hovercraft_maxs_x: f64,
    pub g_hovercraft_maxs_y: f64,
    pub g_hovercraft_mins_x: f64,
    pub g_hovercraft_mins_y: f64,
    pub g_hovercraft_speed_max: f64,
    pub g_hovercraft_steering_car: f64,
    pub g_hovercraft_turn_effectiveness: f64,
    pub g_hovercraft_turn_rate_friction_const: f64,
    pub g_hovercraft_turn_rate_friction_linear: f64,
    pub g_hovercraft_turn_rate_increase: f64,
    pub g_hovercraft_turn_rate_max: f64,
    pub g_hovercraft_turret_offset_chassis_x: f64,
    pub g_hovercraft_turret_offset_chassis_y: f64,
    pub g_hovercraft_turret_offset_turret_x: f64,
    pub g_hovercraft_turret_offset_turret_y: f64,

    pub g_hummer_accel_backward: f64,
    pub g_hummer_accel_forward: f64,
    pub g_hummer_friction_const: f64,
    pub g_hummer_friction_linear: f64,
    pub g_hummer_base_hp: f64,
    pub g_hummer_maxs_x: f64,
    pub g_hummer_maxs_y: f64,
    pub g_hummer_mins_x: f64,
    pub g_hummer_mins_y: f64,
    pub g_hummer_speed_max: f64,
    pub g_hummer_steering_car: f64,
    pub g_hummer_turn_effectiveness: f64,
    pub g_hummer_turn_rate_friction_const: f64,
    pub g_hummer_turn_rate_friction_linear: f64,
    pub g_hummer_turn_rate_increase: f64,
    pub g_hummer_turn_rate_max: f64,
    pub g_hummer_turret_offset_chassis_x: f64,
    pub g_hummer_turret_offset_chassis_y: f64,
    pub g_hummer_turret_offset_turret_x: f64,
    pub g_hummer_turret_offset_turret_y: f64,

    pub g_railgun_beam_duration: f64,
    pub g_railgun_damage: f64,
    pub g_railgun_push: f64,
    pub g_railgun_reload_ammo: u32,
    pub g_railgun_reload_time: f64,
    pub g_railgun_speed: f64,
    pub g_railgun_vehicle_velocity_factor: f64,

    pub g_respawn_delay: f64,

    pub g_rockets_damage_direct: f64,
    pub g_rockets_explosion_damage: f64,
    pub g_rockets_explosion_radius: f64,
    pub g_rockets_explosion_scale: f64,
    pub g_rockets_refire: f64,
    pub g_rockets_reload_ammo: u32,
    pub g_rockets_reload_time: f64,
    pub g_rockets_speed: f64,
    pub g_rockets_vehicle_velocity_factor: f64,

    pub g_self_destruct_damage_center: f64,
    pub g_self_destruct_damage_edge: f64,
    pub g_self_destruct_explosion_scale: f64, // TODO radius
    pub g_self_destruct_radius: f64,

    pub g_tank_accel_backward: f64,
    pub g_tank_accel_forward: f64,
    pub g_tank_friction_const: f64,
    pub g_tank_friction_linear: f64,
    pub g_tank_base_hp: f64,
    pub g_tank_maxs_x: f64,
    pub g_tank_maxs_y: f64,
    pub g_tank_mins_x: f64,
    pub g_tank_mins_y: f64,
    pub g_tank_speed_max: f64,
    pub g_tank_steering_car: f64,
    pub g_tank_turn_effectiveness: f64,
    pub g_tank_turn_rate_friction_const: f64,
    pub g_tank_turn_rate_friction_linear: f64,
    pub g_tank_turn_rate_increase: f64,
    pub g_tank_turn_rate_max: f64,
    pub g_tank_turret_offset_chassis_x: f64,
    pub g_tank_turret_offset_chassis_y: f64,
    pub g_tank_turret_offset_turret_x: f64,
    pub g_tank_turret_offset_turret_y: f64,

    pub g_turret_turn_speed_deg: f64,
    pub g_turret_turn_step_angle_deg: f64,

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

    pub hud_names: bool,
    pub hud_names_alpha: f64,
    pub hud_names_brightness: f64,
    pub hud_names_font_size: f64,
    pub hud_names_shadow_alpha: f64,
    pub hud_names_shadow_mq_x: f32,
    pub hud_names_shadow_mq_y: f32,
    /// NB: these shadows absolutely murder performance in firefox (chromum is ok)
    pub hud_names_shadow_x: f64,
    pub hud_names_shadow_y: f64,
    pub hud_names_x: f64,
    pub hud_names_y: f64,

    pub hud_missile_indicator_dash_length: f64,
    pub hud_missile_indicator_radius: f64,

    pub hud_pause_font_size: f64,
    pub hud_pause_shadow_mq_x: f32,
    pub hud_pause_shadow_mq_y: f32,
    pub hud_pause_shadow_x: f64,
    pub hud_pause_shadow_y: f64,
    pub hud_pause_x: f64,
    pub hud_pause_y: f64,

    pub hud_ranking_font_size: f64,
    pub hud_ranking_shadow_mq_x: f32,
    pub hud_ranking_shadow_mq_y: f32,
    /// Original RW uses 1
    pub hud_ranking_shadow_x: f64,
    /// Original RW uses 1
    pub hud_ranking_shadow_y: f64,
    pub hud_ranking_x: f64,
    pub hud_ranking_y: f64,

    pub hud_score_font_size: f64,
    pub hud_score_shadow_mq_x: f32,
    pub hud_score_shadow_mq_y: f32,
    /// Original RW uses 2
    pub hud_score_shadow_x: f64,
    /// Original RW uses 2
    pub hud_score_shadow_y: f64,
    pub hud_score_x: f64,
    pub hud_score_y: f64,

    pub hud_scoreboard_font_size: f64,
    pub hud_scoreboard_line_height: f64,
    /// NB: these shadows absolutely murder performance in firefox (chromum is ok)
    pub hud_scoreboard_shadow_mq_x: f32,
    pub hud_scoreboard_shadow_mq_y: f32,
    pub hud_scoreboard_shadow_x: f64,
    pub hud_scoreboard_shadow_y: f64,
    pub hud_scoreboard_width_deaths: f32,
    pub hud_scoreboard_width_kills: f32,
    pub hud_scoreboard_width_name: f32,
    pub hud_scoreboard_width_points: f32,

    pub hud_weapon_icon_shadow_alpha: f64,
    pub hud_weapon_icon_shadow_mq_x: f32,
    pub hud_weapon_icon_shadow_mq_y: f32,
    pub hud_weapon_icon_shadow_x: f64,
    pub hud_weapon_icon_shadow_y: f64,
    pub hud_weapon_icon_x: f64,
    pub hud_weapon_icon_y: f64,

    // LATER unify naming - align_to_pixels vs smoothing vs integer_positions
    pub r_align_to_pixels_background: bool,
    pub r_draw_cluster_bombs: bool,
    pub r_explosion_duration: f64,
    pub r_explosions_reverse: bool,
    pub r_smoothing: bool,
    pub r_splitscreen_gap: f64,
    /// This if effectively the oppsite for smoothing but for text.
    pub r_text_integer_positions: bool,

    pub sv_auto_pause_on_minimize: bool,
    pub sv_auto_unpause_on_restore: bool,

    pub sv_gamelogic_mode: TickrateMode,
    pub sv_gamelogic_fixed_fps: f64,
}

#[cfg_attr(feature = "raw_canvas", wasm_bindgen)]
impl Cvars {
    /// Create a new Cvars object with the default RecWars settings.
    pub fn new_rec_wars() -> Self {
        Self::default()
    }

    /// Create a new Cvars object with an approximation of the original RecWar settings.
    pub fn new_rec_war() -> Self {
        Self {
            // This is 15625 tiles - should be more than enough, biggest original maps have 59.
            // Can't use infinity - it would break the math.
            g_railgun_speed: 1_000_000.0,
            ..Self::default()
        }
    }

    /// Reset this Cvars object to the default RecWars settings.
    ///
    /// You can call this from the JavaScript console to change settings on the fly.
    pub fn load_rec_wars(&mut self) {
        *self = Self {
            ..Self::new_rec_wars()
        }
    }

    /// Reset this Cvars object to an approximation of the original RecWar settings.
    ///
    /// You can call this from the JavaScript console to change settings on the fly.
    pub fn load_rec_war(&mut self) {
        *self = Self {
            ..Self::new_rec_war()
        }
    }
}

// Separate impl block without the wasm_bindgen attribute.
// Some fns need to be pub because of the macroquad client,
// however wasm-bindgen would then try to generate bindings for them
// and report errors because they use unsupported types.
// There's no other way to exclude fns from it - e.g. `#[wasm_bindgen(skip)] does nothing.
impl Cvars {
    /// Returns whether the weapon is on the chassis or turret and where relative to that part's center.
    pub(crate) fn g_hardpoint(&self, veh_type: VehicleType, weapon: Weapon) -> (Hardpoint, Vec2f) {
        match veh_type {
            VehicleType::Tank => match weapon {
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
            VehicleType::Hovercraft => match weapon {
                Weapon::Mg => (
                    self.g_hardpoint_hovercraft_machine_gun,
                    Vec2f::new(
                        self.g_hardpoint_hovercraft_machine_gun_x,
                        self.g_hardpoint_hovercraft_machine_gun_y,
                    ),
                ),
                Weapon::Rail => (
                    self.g_hardpoint_hovercraft_railgun,
                    Vec2f::new(
                        self.g_hardpoint_hovercraft_railgun_x,
                        self.g_hardpoint_hovercraft_railgun_y,
                    ),
                ),
                Weapon::Cb => (
                    self.g_hardpoint_hovercraft_cluster_bomb,
                    Vec2f::new(
                        self.g_hardpoint_hovercraft_cluster_bomb_x,
                        self.g_hardpoint_hovercraft_cluster_bomb_y,
                    ),
                ),
                Weapon::Rockets => (
                    self.g_hardpoint_hovercraft_rockets,
                    Vec2f::new(
                        self.g_hardpoint_hovercraft_rockets_x,
                        self.g_hardpoint_hovercraft_rockets_y,
                    ),
                ),
                Weapon::Hm => (
                    self.g_hardpoint_hovercraft_homing_missile,
                    Vec2f::new(
                        self.g_hardpoint_hovercraft_homing_missile_x,
                        self.g_hardpoint_hovercraft_homing_missile_y,
                    ),
                ),
                Weapon::Gm => (
                    self.g_hardpoint_hovercraft_guided_missile,
                    Vec2f::new(
                        self.g_hardpoint_hovercraft_guided_missile_x,
                        self.g_hardpoint_hovercraft_guided_missile_y,
                    ),
                ),
                Weapon::Bfg => (
                    self.g_hardpoint_hovercraft_bfg,
                    Vec2f::new(
                        self.g_hardpoint_hovercraft_bfg_x,
                        self.g_hardpoint_hovercraft_bfg_y,
                    ),
                ),
            },
            VehicleType::Hummer => match weapon {
                Weapon::Mg => (
                    self.g_hardpoint_hummer_machine_gun,
                    Vec2f::new(
                        self.g_hardpoint_hummer_machine_gun_x,
                        self.g_hardpoint_hummer_machine_gun_y,
                    ),
                ),
                Weapon::Rail => (
                    self.g_hardpoint_hummer_railgun,
                    Vec2f::new(
                        self.g_hardpoint_hummer_railgun_x,
                        self.g_hardpoint_hummer_railgun_y,
                    ),
                ),
                Weapon::Cb => (
                    self.g_hardpoint_hummer_cluster_bomb,
                    Vec2f::new(
                        self.g_hardpoint_hummer_cluster_bomb_x,
                        self.g_hardpoint_hummer_cluster_bomb_y,
                    ),
                ),
                Weapon::Rockets => (
                    self.g_hardpoint_hummer_rockets,
                    Vec2f::new(
                        self.g_hardpoint_hummer_rockets_x,
                        self.g_hardpoint_hummer_rockets_y,
                    ),
                ),
                Weapon::Hm => (
                    self.g_hardpoint_hummer_homing_missile,
                    Vec2f::new(
                        self.g_hardpoint_hummer_homing_missile_x,
                        self.g_hardpoint_hummer_homing_missile_y,
                    ),
                ),
                Weapon::Gm => (
                    self.g_hardpoint_hummer_guided_missile,
                    Vec2f::new(
                        self.g_hardpoint_hummer_guided_missile_x,
                        self.g_hardpoint_hummer_guided_missile_y,
                    ),
                ),
                Weapon::Bfg => (
                    self.g_hardpoint_hummer_bfg,
                    Vec2f::new(self.g_hardpoint_hummer_bfg_x, self.g_hardpoint_hummer_bfg_y),
                ),
            },
        }
    }

    pub(crate) fn g_vehicle_hitbox(&self, veh_type: VehicleType) -> Hitbox {
        match veh_type {
            VehicleType::Tank => Hitbox {
                mins: Vec2f::new(self.g_tank_mins_x, self.g_tank_mins_y),
                maxs: Vec2f::new(self.g_tank_maxs_x, self.g_tank_maxs_y),
            },
            VehicleType::Hovercraft => Hitbox {
                mins: Vec2f::new(self.g_hovercraft_mins_x, self.g_hovercraft_mins_y),
                maxs: Vec2f::new(self.g_hovercraft_maxs_x, self.g_hovercraft_maxs_y),
            },
            VehicleType::Hummer => Hitbox {
                mins: Vec2f::new(self.g_hummer_mins_x, self.g_hummer_mins_y),
                maxs: Vec2f::new(self.g_hummer_maxs_x, self.g_hummer_maxs_y),
            },
        }
    }

    pub fn g_vehicle_hp(&self, veh_type: VehicleType) -> f64 {
        let base = match veh_type {
            VehicleType::Tank => self.g_tank_base_hp,
            VehicleType::Hovercraft => self.g_hovercraft_base_hp,
            VehicleType::Hummer => self.g_hummer_base_hp,
        };
        base * self.g_armor
    }

    pub(crate) fn g_vehicle_movement_stats(&self, veh_type: VehicleType) -> MovementStats {
        match veh_type {
            VehicleType::Tank => MovementStats {
                accel_backward: self.g_tank_accel_backward,
                accel_forward: self.g_tank_accel_forward,
                friction_const: self.g_tank_friction_const,
                friction_linear: self.g_tank_friction_linear,
                speed_max: self.g_tank_speed_max,
                steering_car: self.g_tank_steering_car,
                turn_effectiveness: self.g_tank_turn_effectiveness,
                turn_rate_friction_const: self.g_tank_turn_rate_friction_const,
                turn_rate_friction_linear: self.g_tank_turn_rate_friction_linear,
                turn_rate_increase: self.g_tank_turn_rate_increase,
                turn_rate_max: self.g_tank_turn_rate_max,
            },
            VehicleType::Hovercraft => MovementStats {
                accel_backward: self.g_hovercraft_accel_backward,
                accel_forward: self.g_hovercraft_accel_forward,
                friction_const: self.g_hovercraft_friction_const,
                friction_linear: self.g_hovercraft_friction_linear,
                speed_max: self.g_hovercraft_speed_max,
                steering_car: self.g_hovercraft_steering_car,
                turn_effectiveness: self.g_hovercraft_turn_effectiveness,
                turn_rate_friction_const: self.g_hovercraft_turn_rate_friction_const,
                turn_rate_friction_linear: self.g_hovercraft_turn_rate_friction_linear,
                turn_rate_increase: self.g_hovercraft_turn_rate_increase,
                turn_rate_max: self.g_hovercraft_turn_rate_max,
            },
            VehicleType::Hummer => MovementStats {
                accel_backward: self.g_hummer_accel_backward,
                accel_forward: self.g_hummer_accel_forward,
                friction_const: self.g_hummer_friction_const,
                friction_linear: self.g_hummer_friction_linear,
                speed_max: self.g_hummer_speed_max,
                steering_car: self.g_hummer_steering_car,
                turn_effectiveness: self.g_hummer_turn_effectiveness,
                turn_rate_friction_const: self.g_hummer_turn_rate_friction_const,
                turn_rate_friction_linear: self.g_hummer_turn_rate_friction_linear,
                turn_rate_increase: self.g_hummer_turn_rate_increase,
                turn_rate_max: self.g_hummer_turn_rate_max,
            },
        }
    }

    /// Where the turret-chassis connection is on the chassis.
    /// E.g. (0, 0) means the turret rotates around the vehicle's origin.
    pub fn g_vehicle_turret_offset_chassis(&self, veh_type: VehicleType) -> Vec2f {
        match veh_type {
            VehicleType::Tank => Vec2f::new(
                self.g_tank_turret_offset_chassis_x,
                self.g_tank_turret_offset_chassis_y,
            ),
            VehicleType::Hovercraft => Vec2f::new(
                self.g_hovercraft_turret_offset_chassis_x,
                self.g_hovercraft_turret_offset_chassis_y,
            ),
            VehicleType::Hummer => Vec2f::new(
                self.g_hummer_turret_offset_chassis_x,
                self.g_hummer_turret_offset_chassis_y,
            ),
        }
    }

    /// Where the turret-chassis connection is on the turret.
    /// E.g. (0, 0) means the turret rotates around its center.
    pub fn g_vehicle_turret_offset_turret(&self, veh_type: VehicleType) -> Vec2f {
        match veh_type {
            VehicleType::Tank => Vec2f::new(
                self.g_tank_turret_offset_turret_x,
                self.g_tank_turret_offset_turret_y,
            ),
            VehicleType::Hovercraft => Vec2f::new(
                self.g_hovercraft_turret_offset_turret_x,
                self.g_hovercraft_turret_offset_turret_y,
            ),
            VehicleType::Hummer => Vec2f::new(
                self.g_hummer_turret_offset_turret_x,
                self.g_hummer_turret_offset_turret_y,
            ),
        }
    }

    pub(crate) fn g_weapon_damage_direct(&self, weapon: Weapon) -> f64 {
        match weapon {
            Weapon::Mg => self.g_machine_gun_damage,
            Weapon::Rail => self.g_railgun_damage,
            Weapon::Cb => self.g_cluster_bomb_damage_direct,
            Weapon::Rockets => self.g_rockets_damage_direct,
            Weapon::Hm => self.g_homing_missile_damage_direct,
            Weapon::Gm => self.g_guided_missile_damage_direct,
            Weapon::Bfg => self.g_bfg_damage_direct,
        }
    }

    pub(crate) fn g_weapon_explosion_damage(&self, weapon: Weapon) -> f64 {
        match weapon {
            Weapon::Mg => 0.0,
            Weapon::Rail => 0.0,
            Weapon::Cb => self.g_cluster_bomb_explosion_damage,
            Weapon::Rockets => self.g_rockets_explosion_damage,
            Weapon::Hm => self.g_homing_missile_explosion_damage,
            Weapon::Gm => self.g_guided_missile_explosion_damage,
            Weapon::Bfg => self.g_bfg_explosion_damage,
        }
    }

    pub(crate) fn g_weapon_explosion_radius(&self, weapon: Weapon) -> f64 {
        match weapon {
            Weapon::Mg => 0.0,
            Weapon::Rail => 0.0,
            Weapon::Cb => self.g_cluster_bomb_explosion_radius,
            Weapon::Rockets => self.g_rockets_explosion_radius,
            Weapon::Hm => self.g_homing_missile_explosion_radius,
            Weapon::Gm => self.g_guided_missile_explosion_radius,
            Weapon::Bfg => self.g_bfg_explosion_radius,
        }
    }

    pub(crate) fn g_weapon_explosion_scale(&self, weapon: Weapon) -> f64 {
        match weapon {
            Weapon::Mg => 0.0,
            Weapon::Rail => 0.0,
            Weapon::Cb => self.g_cluster_bomb_explosion_scale,
            Weapon::Rockets => self.g_rockets_explosion_scale,
            Weapon::Hm => self.g_homing_missile_explosion_scale,
            Weapon::Gm => self.g_guided_missile_explosion_scale,
            Weapon::Bfg => self.g_bfg_explosion_scale,
        }
    }

    pub(crate) fn g_weapon_movement_stats(&self) -> MovementStats {
        MovementStats {
            accel_backward: 0.0,
            accel_forward: self.g_guided_missile_accel_forward,
            friction_const: self.g_guided_missile_friction_const,
            friction_linear: self.g_guided_missile_friction_linear,
            speed_max: self.g_guided_missile_speed_max,
            steering_car: 0.0,
            turn_effectiveness: self.g_guided_missile_turn_effectiveness,
            turn_rate_friction_const: self.g_guided_missile_turn_rate_friction_const,
            turn_rate_friction_linear: self.g_guided_missile_turn_rate_friction_linear,
            turn_rate_increase: self.g_guided_missile_turn_rate_increase,
            turn_rate_max: self.g_guided_missile_turn_rate_max,
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

    pub fn g_weapon_reload_ammo(&self, weapon: Weapon) -> u32 {
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
            ai: true,

            bots_max: 20,

            d_dbg: false,

            d_draw: true,
            d_draw_crosses: true,
            d_draw_hitboxes: false,
            d_draw_lines: true,
            d_draw_lines_ends_length: 3.0,
            d_draw_perf: true,
            d_draw_text: true,
            d_draw_text_line_height: 14.0,
            d_draw_world_text: true,
            d_explosion_radius: false,
            d_fps: true,
            d_fps_period: 1.0,
            d_fps_x: -300.0,
            d_fps_y: -15.0,
            d_timing_samples: 60,
            d_tracing: false,
            d_seed: 0,
            d_speed: 1.0,

            g_armor: 50.0,

            g_bfg_beam_damage_per_sec: 25.0,
            g_bfg_beam_range: 125.0,
            g_bfg_damage_direct: 0.0,
            g_bfg_explosion_damage: 100.0, // pretty sure from orig RW testing
            g_bfg_explosion_radius: 40.0,
            g_bfg_explosion_scale: 1.0,
            g_bfg_radius: 4.0,
            g_bfg_reload_ammo: 1,
            g_bfg_reload_time: 2.5,
            g_bfg_speed: 150.0,
            g_bfg_vehicle_velocity_factor: 1.0,

            g_cluster_bomb_count: 40,
            g_cluster_bomb_damage_direct: 0.0, // best guess - same as rockets
            g_cluster_bomb_explosion_damage: 25.0,
            g_cluster_bomb_explosion_radius: 20.0,
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
            g_cluster_bomb_vehicle_velocity_factor: 1.0,

            g_ffa_score_kill: 1,
            g_ffa_score_death: -1,

            g_homing_missile_damage_direct: 0.0,
            g_homing_missile_explosion_damage: 56.0, // assumed same as GM
            g_homing_missile_explosion_radius: 40.0,
            g_homing_missile_explosion_scale: 1.0,
            g_homing_missile_reload_ammo: 1,
            g_homing_missile_reload_time: 1.5,
            g_homing_missile_speed_initial: 360.0,
            g_homing_missile_vehicle_velocity_factor: 1.0,

            g_machine_gun_angle_spread: 0.015,
            g_machine_gun_damage: 2.5, // exact from orig RW
            g_machine_gun_refire: 0.050,
            g_machine_gun_reload_ammo: 50,
            g_machine_gun_reload_time: 1.0,
            g_machine_gun_speed: 1000.0,
            g_machine_gun_trail_length: 10.0,
            g_machine_gun_vehicle_velocity_factor: 1.0,

            g_guided_missile_accel_forward: 2000.0,
            g_guided_missile_damage_direct: 0.0,
            g_guided_missile_explosion_damage: 56.0, // exact from orig RW
            g_guided_missile_explosion_radius: 40.0,
            g_guided_missile_explosion_scale: 1.0,
            g_guided_missile_friction_const: 0.0,
            g_guided_missile_friction_linear: 0.99,
            g_guided_missile_reload_ammo: 1,
            g_guided_missile_reload_time: 1.5,
            g_guided_missile_speed_initial: 100.0,
            g_guided_missile_speed_max: f64::INFINITY,
            g_guided_missile_turn_effectiveness: 1.0,
            g_guided_missile_turn_rate_friction_const: 0.10,
            g_guided_missile_turn_rate_friction_linear: 0.995,
            g_guided_missile_turn_rate_increase: 30.0,
            g_guided_missile_turn_rate_max: f64::INFINITY,
            g_guided_missile_vehicle_velocity_factor: 1.0,

            g_hardpoint_hovercraft_machine_gun: Hardpoint::Turret,
            g_hardpoint_hovercraft_machine_gun_x: 19.0,
            g_hardpoint_hovercraft_machine_gun_y: 0.0,
            g_hardpoint_hovercraft_railgun: Hardpoint::Turret,
            g_hardpoint_hovercraft_railgun_x: 19.0,
            g_hardpoint_hovercraft_railgun_y: 0.0,
            g_hardpoint_hovercraft_cluster_bomb: Hardpoint::Turret,
            g_hardpoint_hovercraft_cluster_bomb_x: 19.0,
            g_hardpoint_hovercraft_cluster_bomb_y: 0.0,
            g_hardpoint_hovercraft_rockets: Hardpoint::Turret,
            g_hardpoint_hovercraft_rockets_x: 19.0,
            g_hardpoint_hovercraft_rockets_y: 0.0,
            g_hardpoint_hovercraft_homing_missile: Hardpoint::Chassis,
            g_hardpoint_hovercraft_homing_missile_x: 0.0,
            g_hardpoint_hovercraft_homing_missile_y: -16.0,
            g_hardpoint_hovercraft_guided_missile: Hardpoint::Chassis,
            g_hardpoint_hovercraft_guided_missile_x: 0.0,
            g_hardpoint_hovercraft_guided_missile_y: -16.0,
            g_hardpoint_hovercraft_bfg: Hardpoint::Turret,
            g_hardpoint_hovercraft_bfg_x: 19.0,
            g_hardpoint_hovercraft_bfg_y: 0.0,

            g_hardpoint_hummer_machine_gun: Hardpoint::Chassis,
            g_hardpoint_hummer_machine_gun_x: 10.0,
            g_hardpoint_hummer_machine_gun_y: 9.0,
            g_hardpoint_hummer_railgun: Hardpoint::Chassis,
            g_hardpoint_hummer_railgun_x: 10.0,
            g_hardpoint_hummer_railgun_y: 9.0,
            g_hardpoint_hummer_cluster_bomb: Hardpoint::Turret,
            g_hardpoint_hummer_cluster_bomb_x: 0.0,
            g_hardpoint_hummer_cluster_bomb_y: 0.0,
            g_hardpoint_hummer_rockets: Hardpoint::Turret,
            g_hardpoint_hummer_rockets_x: 0.0,
            g_hardpoint_hummer_rockets_y: 0.0,
            g_hardpoint_hummer_homing_missile: Hardpoint::Chassis,
            g_hardpoint_hummer_homing_missile_x: 0.0,
            g_hardpoint_hummer_homing_missile_y: -10.0,
            g_hardpoint_hummer_guided_missile: Hardpoint::Chassis,
            g_hardpoint_hummer_guided_missile_x: 0.0,
            g_hardpoint_hummer_guided_missile_y: -10.0,
            g_hardpoint_hummer_bfg: Hardpoint::Chassis,
            g_hardpoint_hummer_bfg_x: 10.0,
            g_hardpoint_hummer_bfg_y: 9.0,

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
            g_hardpoint_tank_homing_missile_y: -14.0,
            g_hardpoint_tank_guided_missile: Hardpoint::Chassis,
            g_hardpoint_tank_guided_missile_x: 0.0,
            g_hardpoint_tank_guided_missile_y: -14.0,
            g_hardpoint_tank_bfg: Hardpoint::Turret,
            g_hardpoint_tank_bfg_x: 35.0,
            g_hardpoint_tank_bfg_y: 0.0,

            g_hitcircle_radius: 24.0,

            g_hovercraft_accel_backward: 400.0,
            g_hovercraft_accel_forward: 400.0,
            g_hovercraft_friction_const: 0.0,
            g_hovercraft_friction_linear: 0.6,
            g_hovercraft_base_hp: 0.65,
            g_hovercraft_maxs_x: 22.0,
            g_hovercraft_maxs_y: 14.0,
            g_hovercraft_mins_x: -22.0,
            g_hovercraft_mins_y: -14.0,
            g_hovercraft_speed_max: f64::INFINITY,
            g_hovercraft_steering_car: 0.0,
            g_hovercraft_turn_effectiveness: 0.0,
            g_hovercraft_turn_rate_friction_const: 0.03,
            g_hovercraft_turn_rate_friction_linear: 0.92,
            g_hovercraft_turn_rate_increase: 10.0,
            g_hovercraft_turn_rate_max: f64::INFINITY,
            g_hovercraft_turret_offset_chassis_x: -9.0,
            g_hovercraft_turret_offset_chassis_y: 5.0,
            g_hovercraft_turret_offset_turret_x: -8.0,
            g_hovercraft_turret_offset_turret_y: 0.0,

            g_hummer_accel_backward: 600.0,
            g_hummer_accel_forward: 600.0,
            g_hummer_friction_const: 11.0,
            g_hummer_friction_linear: 0.8,
            g_hummer_base_hp: 0.625,
            g_hummer_maxs_x: 20.0,
            g_hummer_maxs_y: 9.0,
            g_hummer_mins_x: -20.0,
            g_hummer_mins_y: -9.0,
            g_hummer_speed_max: f64::INFINITY,
            g_hummer_steering_car: 200.0,
            g_hummer_turn_effectiveness: 1.0,
            g_hummer_turn_rate_friction_const: 0.04,
            g_hummer_turn_rate_friction_linear: 0.97,
            g_hummer_turn_rate_increase: 18.0,
            g_hummer_turn_rate_max: f64::INFINITY,
            g_hummer_turret_offset_chassis_x: -12.0,
            g_hummer_turret_offset_chassis_y: 0.0,
            g_hummer_turret_offset_turret_x: 0.0,
            g_hummer_turret_offset_turret_y: 0.0,

            g_railgun_beam_duration: 0.05,
            g_railgun_damage: 47.0, // exact from orig RW
            g_railgun_push: 300.0,
            g_railgun_reload_ammo: 1,
            g_railgun_reload_time: 1.0,
            g_railgun_speed: 2000.0,
            g_railgun_vehicle_velocity_factor: 0.0,

            g_respawn_delay: 2.0,

            g_rockets_damage_direct: 25.0, // pretty sure from orig RW testing
            g_rockets_explosion_damage: 0.0,
            g_rockets_explosion_radius: 20.0,
            g_rockets_explosion_scale: 0.5,
            g_rockets_refire: 0.200,
            g_rockets_reload_ammo: 6,
            g_rockets_reload_time: 1.5,
            g_rockets_speed: 600.0,
            g_rockets_vehicle_velocity_factor: 1.0,

            g_self_destruct_damage_center: 150.0,
            g_self_destruct_damage_edge: 0.0,
            g_self_destruct_explosion_scale: 2.0,
            g_self_destruct_radius: 175.0,

            g_tank_accel_backward: 550.0,
            g_tank_accel_forward: 550.0,
            g_tank_friction_const: 50.0,
            g_tank_friction_linear: 0.9,
            g_tank_base_hp: 1.0,
            g_tank_maxs_x: 19.0,
            g_tank_maxs_y: 12.0,
            g_tank_mins_x: -19.0,
            g_tank_mins_y: -12.0,
            g_tank_speed_max: f64::INFINITY,
            g_tank_steering_car: 0.0,
            g_tank_turn_effectiveness: 1.0,
            g_tank_turn_rate_friction_const: 0.05,
            g_tank_turn_rate_friction_linear: 0.96,
            g_tank_turn_rate_increase: 8.0,
            g_tank_turn_rate_max: f64::INFINITY,
            g_tank_turret_offset_chassis_x: -5.0,
            g_tank_turret_offset_chassis_y: 0.0,
            g_tank_turret_offset_turret_x: -14.0,
            g_tank_turret_offset_turret_y: 0.0,

            g_turret_turn_speed_deg: 120.0,
            g_turret_turn_step_angle_deg: 45.0,

            hud_ammo_x: 30.0,
            hud_ammo_y: -30.0,
            hud_ammo_width: 100.0,
            hud_ammo_height: 4.0,

            hud_hp_x: 30.0,
            hud_hp_y: -50.0,
            hud_hp_width: 100.0,
            hud_hp_height: 9.0,

            hud_names: true,
            hud_names_alpha: 1.0,
            hud_names_brightness: 255.0,
            hud_names_font_size: 16.0,
            hud_names_shadow_alpha: 1.0,
            hud_names_shadow_mq_x: 1.0,
            hud_names_shadow_mq_y: 1.0,
            // keep off because of firefox
            hud_names_shadow_x: 0.0,
            hud_names_shadow_y: 0.0,
            hud_names_x: -20.0,
            hud_names_y: 30.0,

            hud_missile_indicator_dash_length: 3.3,
            hud_missile_indicator_radius: 18.0,

            hud_pause_font_size: 64.0,
            hud_pause_shadow_mq_x: 2.0,
            hud_pause_shadow_mq_y: 2.0,
            hud_pause_shadow_x: 0.0,
            hud_pause_shadow_y: 0.0,
            hud_pause_x: 600.0,
            hud_pause_y: 400.0,

            hud_ranking_font_size: 16.0,
            hud_ranking_shadow_mq_x: 1.0,
            hud_ranking_shadow_mq_y: 1.0,
            // keep off because of firefox
            hud_ranking_shadow_x: 0.0,
            hud_ranking_shadow_y: 0.0,
            hud_ranking_x: 80.0,
            hud_ranking_y: -70.0,

            hud_score_font_size: 32.0,
            hud_score_shadow_mq_x: 2.0,
            hud_score_shadow_mq_y: 2.0,
            // keep off because of firefox (even just this one number costs about 5 ms each frame)
            hud_score_shadow_x: 0.0,
            hud_score_shadow_y: 0.0,
            hud_score_x: 30.0,
            hud_score_y: -70.0,

            hud_scoreboard_font_size: 16.0,
            hud_scoreboard_line_height: 18.0,
            hud_scoreboard_shadow_mq_x: 1.0,
            hud_scoreboard_shadow_mq_y: 1.0,
            // keep off because of firefox
            hud_scoreboard_shadow_x: 0.0,
            hud_scoreboard_shadow_y: 0.0,
            hud_scoreboard_width_deaths: 50.0,
            hud_scoreboard_width_kills: 50.0,
            hud_scoreboard_width_name: 150.0,
            hud_scoreboard_width_points: 50.0,

            hud_weapon_icon_shadow_alpha: 0.5,
            hud_weapon_icon_shadow_mq_x: 2.0,
            hud_weapon_icon_shadow_mq_y: 2.0,
            hud_weapon_icon_shadow_x: 2.0,
            hud_weapon_icon_shadow_y: 2.0,
            hud_weapon_icon_x: 170.0,
            hud_weapon_icon_y: -28.0,

            r_align_to_pixels_background: true,
            r_draw_cluster_bombs: true,
            r_explosion_duration: 0.5,
            // After trying true for a while, I think false looks better:
            // - CB looks smoother. With true it sometimes looked like it had 2 stages
            //   because the later explosions were suddenly revealed after the first ones disappeared.
            // - Rockets look better if hitting the same spot.
            r_explosions_reverse: false,
            r_smoothing: false,
            r_splitscreen_gap: 8.0,
            r_text_integer_positions: true,

            sv_auto_pause_on_minimize: true,
            sv_auto_unpause_on_restore: false,

            sv_gamelogic_mode: TickrateMode::Synchronized,
            sv_gamelogic_fixed_fps: 150.0,
        }
    }
}

#[cfg_attr(feature = "raw_canvas", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hardpoint {
    Chassis,
    Turret,
}

/// Various options how to handle different physics/gamelogic and rendering framerates.
#[cfg_attr(feature = "raw_canvas", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickrateMode {
    /// Same FPS as rendering - runs one tick with variable timestep before rendering.
    /// This means simulation always catches up to rendering (wall-clock time) exactly.
    Synchronized,
    /// Fixed FPS - always the same timestep, leftover time carries over to the next render frame.
    /// This means simulation can be only very slightly or up to almost a full frame
    /// behind what should be rendered *and* this delay varries.
    /// As I understand, this can cause a specific kind of stutter called judder.
    Fixed,
    /// Simulation runs in fixed steps as long as it can, the last step is smaller
    /// to catch up to rendering exactly. Next frame, the smaller step is thrown away and simulation
    /// resumes from the last full step so it's deterministic. Too small steps are skipped.
    /// This is described by Jonathan Blow here: https://youtu.be/fdAOPHgW7qM?t=7149
    FixedOrSmaller,
    // There is another option - FixedWithInterpolation:
    // Instead of running with shorter dt to create the intermediate frame whic his thrown away,
    // we'd wait till the next full simulation frame and interpolate to get the intermediate render frame.
    // This would however introduce latency.
    // Also note I believe this would require special handling of events like respawning
    // to avoid interpolating between death and spawn location.
}

#[derive(Debug, Clone)]
pub(crate) struct MovementStats {
    pub(crate) accel_backward: f64,
    pub(crate) accel_forward: f64,
    pub(crate) friction_const: f64,
    pub(crate) friction_linear: f64,
    pub(crate) speed_max: f64,
    pub(crate) steering_car: f64,
    pub(crate) turn_effectiveness: f64,
    pub(crate) turn_rate_friction_const: f64,
    pub(crate) turn_rate_friction_linear: f64,
    pub(crate) turn_rate_increase: f64,
    pub(crate) turn_rate_max: f64,
}

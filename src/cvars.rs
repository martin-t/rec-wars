//! Console variables - configuration options for anything and everything.

use std::num::ParseFloatError;

use cvars::cvars;
use strum_macros::{Display, EnumString};

use crate::prelude::*;

cvars! {
    #![cvars(sorted)]

    //! Console variables - configuration options for anything and everything.
    //!
    //! Prefix meanings:
    //! cl_     client
    //! d_      debugging
    //! g_      gameplay (some of it runs only on the server but this can change with better clientside prediction)
    //! hud_    heads-up display
    //! r_      rendering
    //! sv_     server administration + performance (not gameplay even if it only runs on the server)
    //! sys_    low level / "engine"

    /// Master switch for AI - disable if you want stationary targets
    ai: bool = true,

    /// Final override for the max number of bots
    bots_max: usize = 20,
    /// Desired number of bots based on the number of spawns
    bots_spawns_per_bot: f32 = 1.0,
    /// Desired number of bots based on the number of tiles (map size)
    bots_tiles_per_bot: f32 = 100.0,

    cl_cluster_bomb_size: f64 = 1.5,

    cl_machine_gun_trail_length: f64 = 10.0,
    cl_machine_gun_trail_thickness: f64 = 1.5,

    cl_name1: String = "Player 1".to_owned(),
    cl_name2: String = "Player 2".to_owned(),

    cl_net_connect_retry_delay_ms: u64 = 10,
    cl_net_connect_retry_print_every_n: u32 = 100,
    cl_net_server_addr: String = "127.0.0.1:26000".to_owned(),

    cl_railgun_trail_duration: f64 = 0.05,
    cl_railgun_trail_thickness: f64 = 1.5,

    cl_screenshot_path: String = "screenshots/{date_time}--f{frame_num}-gt{game_time}.tga".to_owned(),
    cl_screenshots: bool = false,

    cl_spawn_indicator_animation_time: f64 = 0.3,
    cl_spawn_indicator_blinking_period: f64 = 0.3,
    cl_spawn_indicator_duration: f64 = 1.5,
    cl_spawn_indicator_square_side_begin: f32 = 800.0,
    cl_spawn_indicator_square_side_end: f32 = 40.0,
    cl_spawn_indicator_thickness: f32 = 2.0,

    /// Two player local multiplayer
    cl_splitscreen: bool = false,

    con_background_alpha: f32 = 0.8,
    con_height_fraction: f32 = 0.45,
    con_history_line_font_size: f32 = 16.0,
    con_history_line_height: f32 = 14.0,
    con_history_x: f32 = 8.0,
    con_history_y_offset: f32 = 25.0,
    con_prompt_group_x: f32 = 16.0,
    con_prompt_group_y_offset: f32 = 26.0,
    con_prompt_label_x: f32 = 8.0,
    con_prompt_label_y_offset: f32 = 22.0,

    /// Master switch for debug output - the d_draw_* group.
    d_draw: bool = true,
    d_draw_arrows: bool = true,
    d_draw_crosses: bool = true,
    d_draw_crosses_half_len: f64 = 5.0,
    /// Sometimes useful if you have trouble finding the crosses.
    d_draw_crosses_line_from_origin: bool = false,
    d_draw_hitboxes: bool = false,
    d_draw_hud: bool = true,
    d_draw_line_thickness: f64 = 1.0,
    d_draw_lines: bool = true,
    /// This sometimes makes it easier to see the lines if they're very short.
    d_draw_lines_ends_half_length: f64 = 5.0,
    d_draw_perf_client: bool = true,
    d_draw_perf_server: bool = true,
    d_draw_rots: bool = true,
    d_draw_rots_size: f64 = 16.0,
    d_draw_text_shadow_alpha: f32 = 0.7,
    d_draw_texts: bool = true,
    d_draw_texts_line_height: f64 = 14.0,
    d_draw_world_texts: bool = true,
    d_examples: bool = true,
    /// During init. Set this first.
    d_exit_on_unknown_cvar: bool = true,
    d_explosion_radius: bool = false,
    /// Draw FPS counter. Intentionally not in the d_draw_* group
    /// so I can easily check perf with and without the other debug output.
    d_fps: bool = true,
    d_fps_period: f64 = 1.0,
    d_fps_x: f64 = -350.0,
    d_fps_y: f64 = -15.0,
    /// Display the last pressed key. Useful for debugging MQ's issues with keyboard layouts.
    d_last_key: bool = false,
    d_log_kills: bool = true,
    d_log_updates_cl: bool = false,
    d_projectiles: bool = false,
    /// The seed to initialize the RNG.
    ///
    /// This is not very helpful by itself because by the time you can change cvars in the console,
    /// the seed has already been used. However, in the macroquad desktop version,
    /// you can set it on the command line.
    ///
    /// If the seed is 0 at match start, the cvar is changed to the current time and that is used as seed.
    /// This means you can look at the cvar's value later and know what seed you need to replay the same game.
    d_seed: u64 = 0,
    /// Change speed of everything in the game
    d_speed: f64 = 1.0,
    d_tickrate_fixed_carry: bool = false,
    d_timing_samples: usize = 60,
    d_tracing: bool = false,

    /// A "temporary" cvar for quick testing. Normally unused but kept here
    /// so I don't have to add a cvar each time I want a quick toggle.
    dbg: bool = false,
    /// Same as dbg but for floats.
    dbgf: f32 = 0.0,
    /// Same as dbg but for ints.
    dbgi: i32 = 0,

    /// Hit points. Recommended values are between 1 and 500, original RecWar used 100 as default.
    ///
    /// Note that the actual number of hitpoints depends on vehicle type, this is just the base value.
    /// By default, the tank uses this value, other vehicles scale it by some multiplier.
    g_armor: f64 = 50.0,

    g_bfg_beam_damage_per_sec: f64 = 25.0,
    g_bfg_beam_range: f64 = 125.0,
    g_bfg_damage_direct: f64 = 0.0,
    g_bfg_explosion_damage: f64 = 100.0, // pretty sure from orig RW testing
    g_bfg_explosion_radius: f64 = 40.0,
    g_bfg_explosion_scale: f64 = 1.0,
    g_bfg_radius: f64 = 4.0,
    g_bfg_reload_ammo: u32 = 1,
    g_bfg_reload_time: f64 = 2.5,
    g_bfg_speed: f64 = 150.0,
    g_bfg_vehicle_velocity_factor: f64 = 1.0,

    g_cluster_bomb_count: i32 = 40,
    g_cluster_bomb_damage_direct: f64 = 0.0, // best guess - same as rockets
    g_cluster_bomb_explosion_damage: f64 = 25.0,
    g_cluster_bomb_explosion_radius: f64 = 20.0,
    g_cluster_bomb_explosion_scale: f64 = 0.5,
    g_cluster_bomb_reload_ammo: u32 = 1,
    g_cluster_bomb_reload_time: f64 = 1.5,
    g_cluster_bomb_shadow_alpha: f64 = 1.0,
    g_cluster_bomb_shadow_x: f64 = 2.0,
    g_cluster_bomb_shadow_y: f64 = 2.0,
    g_cluster_bomb_speed: f64 = 400.0,
    g_cluster_bomb_speed_spread_forward: f64 = 50.0,
    g_cluster_bomb_speed_spread_gaussian: bool = true,
    g_cluster_bomb_speed_spread_sideways: f64 = 50.0,
    g_cluster_bomb_time: f64 = 0.8,
    g_cluster_bomb_time_spread: f64 = 0.2,
    g_cluster_bomb_vehicle_velocity_factor: f64 = 1.0,

    g_ffa_score_death: i32 = -1,
    g_ffa_score_kill: i32 = 1,

    g_guided_missile_accel_forward: f64 = 2000.0,
    g_guided_missile_damage_direct: f64 = 0.0,
    g_guided_missile_explosion_damage: f64 = 56.0, // exact from orig RW
    g_guided_missile_explosion_radius: f64 = 40.0,
    g_guided_missile_explosion_scale: f64 = 1.0,
    g_guided_missile_friction_const: f64 = 0.0,
    g_guided_missile_friction_linear: f64 = 0.99,
    g_guided_missile_reload_ammo: u32 = 1,
    g_guided_missile_reload_time: f64 = 1.5,
    g_guided_missile_speed_initial: f64 = 100.0,
    g_guided_missile_speed_max: f64 = f64::INFINITY,
    g_guided_missile_turn_effectiveness: f64 = 1.0,
    g_guided_missile_turn_rate_friction_const: f64 = 0.10,
    // LATER Interesting bug: setting this to 30.0 and shooting kills everyone on Atrium and gets stuck on other maps.
    //  Same for HM.
    g_guided_missile_turn_rate_friction_linear: f64 = 0.995,
    g_guided_missile_turn_rate_increase: f64 = 30.0,
    g_guided_missile_turn_rate_max: f64 = f64::INFINITY,
    g_guided_missile_vehicle_velocity_factor: f64 = 1.0,

    g_hardpoint_hovercraft_bfg: Hardpoint = Hardpoint::Turret,
    g_hardpoint_hovercraft_bfg_x: f64 = 19.0,
    g_hardpoint_hovercraft_bfg_y: f64 = 0.0,
    g_hardpoint_hovercraft_cluster_bomb: Hardpoint = Hardpoint::Turret,
    g_hardpoint_hovercraft_cluster_bomb_x: f64 = 19.0,
    g_hardpoint_hovercraft_cluster_bomb_y: f64 = 0.0,
    g_hardpoint_hovercraft_guided_missile: Hardpoint = Hardpoint::Chassis,
    g_hardpoint_hovercraft_guided_missile_x: f64 = 0.0,
    g_hardpoint_hovercraft_guided_missile_y: f64 = -16.0,
    g_hardpoint_hovercraft_homing_missile: Hardpoint = Hardpoint::Chassis,
    g_hardpoint_hovercraft_homing_missile_x: f64 = 0.0,
    g_hardpoint_hovercraft_homing_missile_y: f64 = -16.0,
    g_hardpoint_hovercraft_machine_gun: Hardpoint = Hardpoint::Turret,
    g_hardpoint_hovercraft_machine_gun_x: f64 = 19.0,
    g_hardpoint_hovercraft_machine_gun_y: f64 = 0.0,
    g_hardpoint_hovercraft_railgun: Hardpoint = Hardpoint::Turret,
    g_hardpoint_hovercraft_railgun_x: f64 = 19.0,
    g_hardpoint_hovercraft_railgun_y: f64 = 0.0,
    g_hardpoint_hovercraft_rockets: Hardpoint = Hardpoint::Turret,
    g_hardpoint_hovercraft_rockets_x: f64 = 19.0,
    g_hardpoint_hovercraft_rockets_y: f64 = 0.0,

    g_hardpoint_hummer_bfg: Hardpoint = Hardpoint::Chassis,
    g_hardpoint_hummer_bfg_x: f64 = 10.0,
    g_hardpoint_hummer_bfg_y: f64 = 9.0,
    g_hardpoint_hummer_cluster_bomb: Hardpoint = Hardpoint::Turret,
    g_hardpoint_hummer_cluster_bomb_x: f64 = 0.0,
    g_hardpoint_hummer_cluster_bomb_y: f64 = 0.0,
    g_hardpoint_hummer_guided_missile: Hardpoint = Hardpoint::Chassis,
    g_hardpoint_hummer_guided_missile_x: f64 = 0.0,
    g_hardpoint_hummer_guided_missile_y: f64 = -10.0,
    g_hardpoint_hummer_homing_missile: Hardpoint = Hardpoint::Chassis,
    g_hardpoint_hummer_homing_missile_x: f64 = 0.0,
    g_hardpoint_hummer_homing_missile_y: f64 = -10.0,
    g_hardpoint_hummer_machine_gun: Hardpoint = Hardpoint::Chassis,
    g_hardpoint_hummer_machine_gun_x: f64 = 10.0,
    g_hardpoint_hummer_machine_gun_y: f64 = 9.0,
    g_hardpoint_hummer_railgun: Hardpoint = Hardpoint::Chassis,
    g_hardpoint_hummer_railgun_x: f64 = 10.0,
    g_hardpoint_hummer_railgun_y: f64 = 9.0,
    g_hardpoint_hummer_rockets: Hardpoint = Hardpoint::Turret,
    g_hardpoint_hummer_rockets_x: f64 = 0.0,
    g_hardpoint_hummer_rockets_y: f64 = 0.0,

    g_hardpoint_tank_bfg: Hardpoint = Hardpoint::Turret,
    g_hardpoint_tank_bfg_x: f64 = 35.0,
    g_hardpoint_tank_bfg_y: f64 = 0.0,
    g_hardpoint_tank_cluster_bomb: Hardpoint = Hardpoint::Turret,
    g_hardpoint_tank_cluster_bomb_x: f64 = 35.0,
    g_hardpoint_tank_cluster_bomb_y: f64 = 0.0,
    g_hardpoint_tank_guided_missile: Hardpoint = Hardpoint::Chassis,
    g_hardpoint_tank_guided_missile_x: f64 = 0.0,
    g_hardpoint_tank_guided_missile_y: f64 = -14.0,
    g_hardpoint_tank_homing_missile: Hardpoint = Hardpoint::Chassis,
    g_hardpoint_tank_homing_missile_x: f64 = 0.0,
    g_hardpoint_tank_homing_missile_y: f64 = -14.0,
    g_hardpoint_tank_machine_gun: Hardpoint = Hardpoint::Turret,
    g_hardpoint_tank_machine_gun_x: f64 = 12.0,
    g_hardpoint_tank_machine_gun_y: f64 = -5.0,
    g_hardpoint_tank_railgun: Hardpoint = Hardpoint::Turret,
    g_hardpoint_tank_railgun_x: f64 = 35.0,
    g_hardpoint_tank_railgun_y: f64 = 0.0,
    g_hardpoint_tank_rockets: Hardpoint = Hardpoint::Turret,
    g_hardpoint_tank_rockets_x: f64 = 35.0,
    g_hardpoint_tank_rockets_y: f64 = 0.0,

    g_hitcircle_radius: f64 = 24.0, // LATER proper hitbox

    g_homing_missile_accel_forward: f64 = 2000.0,
    g_homing_missile_angle_detect: f64 = 40.0f64.to_radians(), // LATER (also other places) use Deg
    g_homing_missile_angle_forget: f64 = 50.0f64.to_radians(),
    g_homing_missile_damage_direct: f64 = 0.0,
    g_homing_missile_explosion_damage: f64 = 56.0, // assumed same as GM
    g_homing_missile_explosion_radius: f64 = 40.0,
    g_homing_missile_explosion_scale: f64 = 1.0,
    g_homing_missile_friction_const: f64 = 0.0,
    g_homing_missile_friction_linear: f64 = 0.99,
    g_homing_missile_reload_ammo: u32 = 1,
    g_homing_missile_reload_time: f64 = 1.5,
    g_homing_missile_speed_initial: f64 = 100.0,
    g_homing_missile_speed_max: f64 = f64::INFINITY,
    g_homing_missile_turn_effectiveness: f64 = 1.0,
    g_homing_missile_turn_rate_friction_const: f64 = 0.10,
    g_homing_missile_turn_rate_friction_linear: f64 = 0.995,
    g_homing_missile_turn_rate_increase: f64 = 10.0,
    g_homing_missile_turn_rate_max: f64 = f64::INFINITY,
    g_homing_missile_vehicle_velocity_factor: f64 = 1.0,

    g_hovercraft_accel_backward: f64 = 400.0,
    g_hovercraft_accel_forward: f64 = 400.0,
    g_hovercraft_armor_scale: f64 = 0.65,
    g_hovercraft_friction_const: f64 = 0.0,
    g_hovercraft_friction_linear: f64 = 0.6,
    g_hovercraft_maxs_x: f64 = 22.0,
    g_hovercraft_maxs_y: f64 = 14.0,
    g_hovercraft_mins_x: f64 = -22.0,
    g_hovercraft_mins_y: f64 = -14.0,
    g_hovercraft_speed_max: f64 = f64::INFINITY,
    g_hovercraft_steering_car: f64 = 0.0,
    g_hovercraft_turn_effectiveness: f64 = 0.0,
    g_hovercraft_turn_rate_friction_const: f64 = 0.03,
    g_hovercraft_turn_rate_friction_linear: f64 = 0.92,
    g_hovercraft_turn_rate_increase: f64 = 10.0,
    g_hovercraft_turn_rate_max: f64 = f64::INFINITY,
    g_hovercraft_turret_offset_chassis_x: f64 = -9.0,
    g_hovercraft_turret_offset_chassis_y: f64 = 5.0,
    g_hovercraft_turret_offset_turret_x: f64 = -8.0,
    g_hovercraft_turret_offset_turret_y: f64 = 0.0,

    g_hummer_accel_backward: f64 = 600.0,
    g_hummer_accel_forward: f64 = 600.0,
    g_hummer_armor_scale: f64 = 0.625,
    g_hummer_friction_const: f64 = 11.0,
    g_hummer_friction_linear: f64 = 0.8,
    g_hummer_maxs_x: f64 = 20.0,
    g_hummer_maxs_y: f64 = 9.0,
    g_hummer_mins_x: f64 = -20.0,
    g_hummer_mins_y: f64 = -9.0,
    g_hummer_speed_max: f64 = f64::INFINITY,
    g_hummer_steering_car: f64 = 200.0,
    g_hummer_turn_effectiveness: f64 = 1.0,
    g_hummer_turn_rate_friction_const: f64 = 0.04,
    g_hummer_turn_rate_friction_linear: f64 = 0.97,
    g_hummer_turn_rate_increase: f64 = 18.0,
    g_hummer_turn_rate_max: f64 = f64::INFINITY,
    g_hummer_turret_offset_chassis_x: f64 = -12.0,
    g_hummer_turret_offset_chassis_y: f64 = 0.0,
    g_hummer_turret_offset_turret_x: f64 = 0.0,
    g_hummer_turret_offset_turret_y: f64 = 0.0,

    g_machine_gun_angle_spread: f64 = 0.015,
    g_machine_gun_damage: f64 = 2.5, // exact from orig RW
    g_machine_gun_refire: f64 = 0.050,
    g_machine_gun_reload_ammo: u32 = 50,
    g_machine_gun_reload_time: f64 = 1.0,
    g_machine_gun_speed: f64 = 1000.0,
    g_machine_gun_vehicle_velocity_factor: f64 = 1.0,

    /// The map to play on. Set to empty string for random.
    g_map: String = "".to_owned(),

    g_players_max: usize = 64,
    g_players_min: usize = 4,

    g_railgun_damage: f64 = 47.0, // exact from orig RW
    g_railgun_push: f64 = 300.0,
    g_railgun_reload_ammo: u32 = 1,
    g_railgun_reload_time: f64 = 1.0,
    g_railgun_speed: f64 = 2500.0,
    g_railgun_vehicle_velocity_factor: f64 = 0.0,

    g_respawn_delay: f64 = 0.5, // LATER this used to be 2 s, did RW use 2 s?

    g_rockets_damage_direct: f64 = 25.0,
    g_rockets_explosion_damage: f64 = 0.0, // pretty sure from orig RW testing
    g_rockets_explosion_radius: f64 = 20.0,
    g_rockets_explosion_scale: f64 = 0.5,
    g_rockets_refire: f64 = 0.200,
    g_rockets_reload_ammo: u32 = 6,
    g_rockets_reload_time: f64 = 1.5,
    g_rockets_speed: f64 = 600.0,
    g_rockets_vehicle_velocity_factor: f64 = 1.0,

    g_self_destruct_damage_center: f64 = 150.0,
    g_self_destruct_damage_edge: f64 = 0.0,
    g_self_destruct_explosion_scale: f64 = 2.0, // LATER radius
    g_self_destruct_radius: f64 = 175.0,

    g_tank_accel_backward: f64 = 550.0,
    g_tank_accel_forward: f64 = 550.0,
    g_tank_armor_scale: f64 = 1.0,
    g_tank_friction_const: f64 = 50.0,
    g_tank_friction_linear: f64 = 0.9,
    g_tank_maxs_x: f64 = 19.0,
    g_tank_maxs_y: f64 = 12.0,
    g_tank_mins_x: f64 = -19.0,
    g_tank_mins_y: f64 = -12.0,
    g_tank_speed_max: f64 = f64::INFINITY,
    g_tank_steering_car: f64 = 0.0,
    g_tank_turn_effectiveness: f64 = 1.0,
    g_tank_turn_rate_friction_const: f64 = 0.05,
    g_tank_turn_rate_friction_linear: f64 = 0.96,
    g_tank_turn_rate_increase: f64 = 8.0,
    g_tank_turn_rate_max: f64 = f64::INFINITY,
    g_tank_turret_offset_chassis_x: f64 = -5.0,
    g_tank_turret_offset_chassis_y: f64 = 0.0,
    g_tank_turret_offset_turret_x: f64 = -14.0,
    g_tank_turret_offset_turret_y: f64 = 0.0,

    g_turret_turn_speed_deg: f64 = 120.0,
    g_turret_turn_step_angle_deg: f64 = 45.0,

    /// Original RecWar had 4.
    hud_ammo_height: f64 = 4.0,
    /// Original RecWar had 99.
    hud_ammo_width: f64 = 100.0,
    hud_ammo_x: f64 = 30.0,
    hud_ammo_y: f64 = -30.0,

    /// Original RecWar had 9.
    hud_hp_height: f64 = 9.0,
    /// Original RecWar had 99.
    hud_hp_width: f64 = 100.0,
    hud_hp_x: f64 = 30.0,
    hud_hp_y: f64 = -50.0,

    hud_missile_indicator_dash_length: f64 = 3.3,
    hud_missile_indicator_radius: f64 = 18.0,

    hud_names: bool = true,
    hud_names_alpha: f64 = 1.0,
    hud_names_brightness: f64 = 255.0,
    hud_names_font_size: f64 = 16.0,
    hud_names_shadow_alpha: f32 = 1.0,
    hud_names_shadow_x: f32 = 1.0,
    hud_names_shadow_y: f32 = 1.0,
    hud_names_x: f64 = -20.0,
    hud_names_y: f64 = 30.0,

    hud_notifications_alpha_old: f32 = 0.5,
    hud_notifications_color_death: CVec3 = CVec3::RED,
    hud_notifications_color_kill: CVec3 = CVec3::BLUE2,
    hud_notifications_duration: f64 = 3.0,
    hud_notifications_duration_fade_out: f64 = 0.25,
    hud_notifications_duration_grow: f64 = 0.02,
    hud_notifications_duration_large: f64 = 0.03,
    hud_notifications_duration_shrink: f64 = 0.06,
    hud_notifications_font_size: f64 = 24.0,
    hud_notifications_font_size_large: f64 = 28.0,
    hud_notifications_y_from_center: f32 = -250.0,
    hud_notifications_y_from_top: f32 = 150.0,
    hud_notifications_y_offset: f32 = -40.0,

    hud_pause_font_size: f64 = 64.0,
    hud_pause_shadow_x: f32 = 2.0,
    hud_pause_shadow_y: f32 = 2.0,

    hud_perf_client_x: f64 = -250.0,
    hud_perf_client_y: f64 = -105.0,
    hud_perf_server_x: f64 = -500.0,
    hud_perf_server_y: f64 = -105.0,

    hud_ranking_font_size: f64 = 16.0,
    /// Original RW uses 1
    hud_ranking_shadow_x: f32 = 1.0,
    /// Original RW uses 1
    hud_ranking_shadow_y: f32 = 1.0,
    hud_ranking_x: f64 = 80.0,
    hud_ranking_y: f64 = -70.0,

    hud_score_font_size: f64 = 32.0,
    /// Original RW uses 2
    hud_score_shadow_x: f32 = 2.0,
    /// Original RW uses 2
    hud_score_shadow_y: f32 = 2.0,
    hud_score_x: f64 = 30.0,
    hud_score_y: f64 = -70.0,

    hud_scoreboard_font_size: f64 = 16.0,
    hud_scoreboard_line_height: f64 = 18.0,
    /// NB: these shadows absolutely murder performance in firefox (chromum is ok)
    hud_scoreboard_shadow_x: f32 = 1.0,
    hud_scoreboard_shadow_y: f32 = 1.0,
    hud_scoreboard_width_deaths: f32 = 50.0,
    hud_scoreboard_width_kills: f32 = 50.0,
    hud_scoreboard_width_name: f32 = 150.0,
    hud_scoreboard_width_points: f32 = 50.0,

    hud_weapon_icon_shadow_alpha: f64 = 0.5,
    hud_weapon_icon_shadow_x: f32 = 2.0,
    hud_weapon_icon_shadow_y: f32 = 2.0,
    hud_weapon_icon_x: f64 = 170.0,
    hud_weapon_icon_y: f64 = -28.0,

    /// This is in a way the opposite of smoothing
    r_align_to_pixels_background: bool = true,
    r_align_to_pixels_text: bool = true,
    r_cluster_bombs: bool = true,
    r_explosion_duration: f64 = 0.5,
    // After trying true for a while, I think false looks better:
    // - CB looks smoother. With true it sometimes looked like it had 2 stages
    //   because the later explosions were suddenly revealed after the first ones disappeared.
    // - Rockets look better if hitting the same spot.
    r_explosions_reverse_order: bool = false,
    r_guided_missile_offset_x: f64 = 5.0,
    r_guided_missile_offset_y: f64 = 0.0,
    r_homing_missile_offset_x: f64 = 5.0,
    r_homing_missile_offset_y: f64 = 0.0,
    r_rockets_offset_x: f64 = 5.0,
    r_rockets_offset_y: f64 = 0.0,
    r_smoothing: bool = false,
    r_splitscreen_gap: f64 = 8.0,

    /// LATER fix - Does not work in MQ: https://github.com/not-fl3/macroquad/issues/264
    sv_auto_pause_on_minimize: bool = true,
    /// LATER fix - Does not work in MQ: https://github.com/not-fl3/macroquad/issues/264
    sv_auto_unpause_on_restore: bool = false,

    sv_net_listen_addr: String = "127.0.0.1:26000".to_owned(),

    /// LATER Without extrapolation, this needs to be significantly higher than framerate to avoid judder.
    ///     Assuming rendering at 60 fps:
    ///     With 30 updates, it's easily visible on vehicle movement.
    ///     With 60, it's sometimes still noticeable on vehicles but mostly on moving text (names) being less readable.
    sys_tickrate_fixed_fps: f64 = 150.0,
    sys_tickrate_mode: TickrateMode = TickrateMode::Fixed,
}

impl Cvars {
    /// Create a new Cvars object with the default RecWars settings.
    pub fn new_rec_wars() -> Self {
        Self::default()
    }

    /// Create a new Cvars object with an approximation of the original RecWar settings.
    #[allow(dead_code)] // LATER allow using balance presets again
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
    /// LATER Allow calling this from the console to change settings on the fly.
    #[allow(dead_code)]
    pub fn load_rec_wars(&mut self) {
        *self = Self {
            ..Self::new_rec_wars()
        }
    }

    /// Reset this Cvars object to an approximation of the original RecWar settings.
    ///
    /// LATER Allow calling this from the console to change settings on the fly.
    #[allow(dead_code)]
    pub fn load_rec_war(&mut self) {
        *self = Self {
            ..Self::new_rec_war()
        }
    }

    /// Returns whether the weapon is on the chassis or turret and where relative to that part's center.
    pub fn g_hardpoint(&self, veh_type: VehicleType, weapon: Weapon) -> (Hardpoint, Vec2f) {
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

    pub fn g_vehicle_hitbox(&self, veh_type: VehicleType) -> Hitbox {
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
        let scale = match veh_type {
            VehicleType::Tank => self.g_tank_armor_scale,
            VehicleType::Hovercraft => self.g_hovercraft_armor_scale,
            VehicleType::Hummer => self.g_hummer_armor_scale,
        };
        self.g_armor * scale
    }

    pub fn g_vehicle_movement_stats(&self, veh_type: VehicleType) -> MovementStats {
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

    pub fn g_weapon_damage_direct(&self, weapon: Weapon) -> f64 {
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

    pub fn g_weapon_explosion_damage(&self, weapon: Weapon) -> f64 {
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

    pub fn g_weapon_explosion_radius(&self, weapon: Weapon) -> f64 {
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

    pub fn g_weapon_explosion_scale(&self, weapon: Weapon) -> f64 {
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

    pub fn g_homing_missile_movement_stats(&self) -> MovementStats {
        MovementStats {
            accel_backward: 0.0,
            accel_forward: self.g_homing_missile_accel_forward,
            friction_const: self.g_homing_missile_friction_const,
            friction_linear: self.g_homing_missile_friction_linear,
            speed_max: self.g_homing_missile_speed_max,
            steering_car: 0.0,
            turn_effectiveness: self.g_homing_missile_turn_effectiveness,
            turn_rate_friction_const: self.g_homing_missile_turn_rate_friction_const,
            turn_rate_friction_linear: self.g_homing_missile_turn_rate_friction_linear,
            turn_rate_increase: self.g_homing_missile_turn_rate_increase,
            turn_rate_max: self.g_homing_missile_turn_rate_max,
        }
    }

    pub fn g_guided_missile_movement_stats(&self) -> MovementStats {
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

    pub fn g_weapon_refire(&self, weapon: Weapon) -> f64 {
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

    pub fn g_weapon_reload_time(&self, weapon: Weapon) -> f64 {
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

/// Vec3 with support for cvars. Should be converted to Vec3 before use in gamecode.
#[derive(Debug, Clone, Copy)]
pub struct CVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[allow(dead_code)]
impl CVec3 {
    const RED: Self = Self::new(1.0, 0.0, 0.0);
    const GREEN: Self = Self::new(0.0, 1.0, 0.0);
    const BLUE: Self = Self::new(0.0, 0.0, 1.0);
    const BLUE2: Self = Self::new(0.0, 0.2, 1.0);
    const WHITE: Self = Self::new(1.0, 1.0, 1.0);
    const BLACK: Self = Self::new(0.0, 0.0, 0.0);
    const YELLOW: Self = Self::new(1.0, 1.0, 0.0);
    const MAGENTA: Self = Self::new(1.0, 0.0, 1.0);
    const CYAN: Self = Self::new(0.0, 1.0, 1.0);

    const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl FromStr for CVec3 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_cvec3(s).map_err(|e| {
            if let Some(e) = e {
                format!("Expected format `'x y z'`, got `{}`: {}", s, e)
            } else {
                format!("Expected format `'x y z'`, got `{}`", s)
            }
        })
    }
}

fn parse_cvec3(mut s: &str) -> Result<CVec3, Option<ParseFloatError>> {
    if s.starts_with('\'') && s.ends_with('\'') {
        s = &s[1..s.len() - 1];
    }
    let mut parts = s.split(' ');
    let x = parts.next().ok_or(None)?.parse().map_err(Some)?;
    let y = parts.next().ok_or(None)?.parse().map_err(Some)?;
    let z = parts.next().ok_or(None)?.parse().map_err(Some)?;
    if parts.next().is_some() {
        return Err(None);
    }
    Ok(CVec3::new(x, y, z))
}

impl Display for CVec3 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "'{:?} {:?} {:?}'", self.x, self.y, self.z)
    }
}

impl From<CVec3> for Color {
    fn from(v: CVec3) -> Self {
        Color::new(v.x, v.y, v.z, 1.0)
    }
}

impl From<Color> for CVec3 {
    fn from(c: Color) -> Self {
        soft_assert_eq!(c.a, 1.0);
        CVec3::new(c.r, c.g, c.b)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
#[strum(ascii_case_insensitive)]
pub enum Hardpoint {
    Chassis,
    Turret,
}

/// Various options how to handle different physics/gamelogic and rendering framerates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
#[strum(ascii_case_insensitive)]
pub enum TickrateMode {
    /// Same FPS as rendering - runs one tick with variable timestep before rendering.
    /// This means simulation always catches up to rendering (wall-clock time) exactly.
    Variable,
    /// Fixed FPS - always the same timestep, leftover time carries over to the next render frame.
    /// This means simulation can be only very slightly or up to almost a full frame
    /// behind what should be rendered *and* this delay varries.
    /// As I understand, this can cause a specific kind of stutter called judder.
    Fixed,
    // /// Simulation runs in fixed steps as long as it can, the last step is smaller
    // /// to catch up to rendering exactly. Next frame, the smaller step is thrown away and simulation
    // /// resumes from the last full step so it's deterministic. Too small steps are skipped.
    // /// This is described by Jonathan Blow here: https://youtu.be/fdAOPHgW7qM?t=7149
    // FixedWithExtrapolation,
    // There is another option - FixedWithInterpolation:
    // Instead of running with shorter dt to create the intermediate frame which is thrown away,
    // we'd wait till the next full simulation frame and interpolate to get the intermediate render frame.
    // This would however introduce latency.
    // Also note I believe this would require special handling of events like respawning
    // to avoid interpolating between death and spawn location.
}

#[derive(Debug, Clone)]
pub struct MovementStats {
    pub accel_backward: f64,
    pub accel_forward: f64,
    pub friction_const: f64,
    pub friction_linear: f64,
    pub speed_max: f64,
    pub steering_car: f64,
    pub turn_effectiveness: f64,
    pub turn_rate_friction_const: f64,
    pub turn_rate_friction_linear: f64,
    pub turn_rate_increase: f64,
    pub turn_rate_max: f64,
}

//pub fn load_cvars

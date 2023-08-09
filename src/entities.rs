//! The E and C in ECS
//!
//! We're using the ECS design pattern (decouple behavior from data),
//! just without the ECS data structure (we use generational arenas instead).
//! Most game data goes here - entities are structs, components are fields.
//!
//! Some entities have pure member functions.
//! This is not a violation of the ECS pattern,
//! because they don't modify game state - they're not behavior.

use strum_macros::{EnumCount, FromRepr};

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    /// NOTE about potential bugs when refactoring:
    /// - vehicle can move while dead (this is a classic at this point)
    /// - can guide missile while dead
    /// - can guide multiple missiles (LATER optionally allow by cvar)
    /// - missile input is not reset after death / launching another (results in flying in circles)
    /// - missile stops after player dies / launches another
    pub input: Input,
    pub respawn: Respawn,
    pub death_time: f64,
    pub vehicle: Option<Index>,
    pub guided_missile: Option<Index>,
    pub cur_weapon: Weapon,
    pub score: Score,
    pub notifications: Vec<Notification>,
}

impl Player {
    pub fn new(name: String) -> Self {
        Self {
            name,
            input: Input::new(),
            respawn: Respawn::No,
            death_time: 0.0,
            vehicle: None,
            guided_missile: None,
            cur_weapon: Weapon::Mg,
            score: Score::default(),
            notifications: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Respawn {
    No,
    Pressed,
    Scheduled,
}

#[derive(Debug, Clone, Default)]
pub struct Score {
    pub kills: i32,
    pub deaths: i32,
    pub suicides: i32,
}

impl Score {
    pub fn points(&self, cvars: &Cvars) -> i32 {
        self.kills * cvars.g_ffa_score_kill + self.deaths * cvars.g_ffa_score_death
    }
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub text: String,
    pub color: CVec3,
    pub start_time: f64,
}

impl Notification {
    pub fn new(text: String, color: CVec3, start_time: f64) -> Self {
        Self {
            text,
            color,
            start_time,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ai {
    pub player: Index,
    pub movement: i32,
    pub turning: i32,
    pub firing: bool,
}

impl Ai {
    pub fn new(player: Index) -> Self {
        Self {
            player,
            movement: 0,
            turning: 0,
            firing: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Vehicle {
    pub pos: Vec2f,
    pub vel: Vec2f,
    pub angle: f64,
    pub turn_rate: f64,
    pub veh_type: VehicleType,
    pub hitbox: Hitbox,
    /// Angle from vehicle, see Coord system for more
    pub turret_angle_current: f64,
    pub turret_angle_wanted: f64,
    /// HP between 0 and 1 - saving the fraction here instead of absolute hit points so armor cvars can be adjusted during a match.
    pub hp_fraction: f64,
    /// Each weapon has a separate reload status even if they all reload at the same time.
    /// I plan to generalize this and have a cvar to choose between multiple reload mechanisms.
    pub ammos: Vec<Ammo>,
    pub spawn_time: f64,
    pub owner: Index,
    /// Indices of homing missiles targeting this vehicle.
    pub hms: Vec<Index>,
}

impl Vehicle {
    #[must_use]
    pub fn new(
        cvars: &Cvars,
        pos: Vec2f,
        angle: f64,
        veh_type: VehicleType,
        spawn_time: f64,
        owner: Index,
    ) -> Vehicle {
        let hitbox = cvars.g_vehicle_hitbox(veh_type);
        let ammos = vec![
            Ammo::Loaded(0.0, cvars.g_weapon_reload_ammo(Weapon::Mg)),
            Ammo::Loaded(0.0, cvars.g_weapon_reload_ammo(Weapon::Rail)),
            Ammo::Loaded(0.0, cvars.g_weapon_reload_ammo(Weapon::Cb)),
            Ammo::Loaded(0.0, cvars.g_weapon_reload_ammo(Weapon::Rockets)),
            Ammo::Loaded(0.0, cvars.g_weapon_reload_ammo(Weapon::Hm)),
            Ammo::Loaded(0.0, cvars.g_weapon_reload_ammo(Weapon::Gm)),
            Ammo::Loaded(0.0, cvars.g_weapon_reload_ammo(Weapon::Bfg)),
        ];

        Vehicle {
            pos,
            vel: Vec2f::zero(),
            angle,
            turn_rate: 0.0,
            veh_type,
            hitbox,
            turret_angle_current: 0.0,
            turret_angle_wanted: 0.0,
            hp_fraction: 1.0,
            ammos,
            spawn_time,
            owner,
            hms: Vec::new(),
        }
    }

    pub fn destroyed(&self) -> bool {
        self.hp_fraction <= 0.0
    }
}

#[derive(Debug, Clone, Copy, FromRepr)]
pub enum VehicleType {
    Tank,
    Hovercraft,
    Hummer,
}

#[derive(Debug, Clone, Copy)]
pub enum Ammo {
    /// Refire delay end time, ammo count remaining
    Loaded(f64, u32),
    /// Start time, end time
    Reloading(f64, f64),
}

/// A projectile fired by a vehicle.
///
/// All weapons share the same projectile struct
/// so long-term it's possible to add/modify/remove weapon behavior dynamically.
///
/// Every weapon shoots a projectile, there is no hitscan.
/// If you want hitscan behavior, use a high (but finite) projectile speed.
#[derive(Debug, Clone)]
pub struct Projectile {
    pub weapon: Weapon,
    pub pos: Vec2f,
    pub vel: Vec2f,
    pub angle: f64,
    pub turn_rate: f64,
    pub explode_time: f64,
    pub owner: Index,
    pub target: Option<Index>,
}

/// Weapon type - currently hardcoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumCount, FromRepr)]
pub enum Weapon {
    Mg,
    Rail,
    Cb,
    Rockets,
    Hm,
    Gm,
    Bfg,
}

#[derive(Debug, Clone, Copy)]
pub struct Hitbox {
    pub mins: Vec2f,
    pub maxs: Vec2f,
}

impl Hitbox {
    pub fn corners(self, pos: Vec2f, angle: f64) -> [Vec2f; 4] {
        let back_left = pos + Vec2f::new(self.mins.x, self.mins.y).rotated_z(angle);
        let front_left = pos + Vec2f::new(self.maxs.x, self.mins.y).rotated_z(angle);
        let front_right = pos + Vec2f::new(self.maxs.x, self.maxs.y).rotated_z(angle);
        let back_right = pos + Vec2f::new(self.mins.x, self.maxs.y).rotated_z(angle);
        [back_left, front_left, front_right, back_right]
    }
}

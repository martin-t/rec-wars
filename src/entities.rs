//! The E and C in ECS
//!
//! We're using the ECS design pattern (decouple behavior from data),
//! just without the ECS data structure (we use generational arenas instead).
//! Most game data goes here - entities are structs, components are fields.
//!
//! Some entities have pure member functions.
//! This is not a violation of the ECS pattern,
//! because they don't modify game state - they're not behavior.

use enumn::N;
use thunderdome::Index;

use crate::{cvars::Cvars, game_state::Input, map::Vec2f};

#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    /// NOTE about potential bugs when refactoring:
    /// - vehicle can move while dead (this is a classic at this point)
    /// - can guide missile while dead
    /// - can guide multiple missiles (LATER optionally allow by cvar)
    /// - missile input is not reset after death / launching another (results in flying in circles)
    /// - missile stops after player dies / launches another
    pub(crate) input: Input,
    pub(crate) respawn: Respawn,
    pub(crate) death_time: f64,
    pub vehicle: Option<Index>,
    pub guided_missile: Option<Index>,
    pub cur_weapon: Weapon,
    pub score: Score,
}

impl Player {
    // LATER pub(crate)?
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
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Respawn {
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

#[derive(Debug, Clone)]
pub(crate) struct Ai {
    pub(crate) player: Index,
    pub(crate) movement: i32,
    pub(crate) turning: i32,
    pub(crate) firing: bool,
}

impl Ai {
    pub(crate) fn new(player: Index) -> Self {
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
    pub(crate) vel: Vec2f,
    pub angle: f64,
    pub(crate) turn_rate: f64,
    pub veh_type: VehicleType,
    pub(crate) hitbox: Hitbox,
    /// Angle from vehicle, see Coord system for more
    pub turret_angle_current: f64,
    pub(crate) turret_angle_wanted: f64,
    /// HP between 0 and 1 - saving the fraction here instead of absolute hit points so armor cvars can be adjusted during a match.
    pub(crate) hp_fraction: f64,
    /// Each weapon has a separate reload status even if they all reload at the same time.
    /// I plan to generalize this and have a cvar to choose between multiple reload mechanisms.
    pub(crate) ammos: Vec<Ammo>,
    pub(crate) spawn_time: f64,
    pub owner: Index,
}

impl Vehicle {
    #[must_use]
    pub(crate) fn new(
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
        }
    }

    pub fn destroyed(&self) -> bool {
        self.hp_fraction <= 0.0
    }
}

#[derive(Debug, Clone, Copy, N)]
pub enum VehicleType {
    Tank,
    Hovercraft,
    Hummer,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Ammo {
    /// Refire delay end time, ammo count remaining
    Loaded(f64, u32),
    /// Start time, end time
    Reloading(f64, f64),
}

#[derive(Debug, Clone)]
pub struct Projectile {
    pub weapon: Weapon,
    pub pos: Vec2f,
    pub vel: Vec2f,
    pub angle: f64,
    pub(crate) turn_rate: f64,
    pub(crate) explode_time: f64,
    pub(crate) owner: Index,
}

pub(crate) const WEAPS_CNT: u8 = 7;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, N)]
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
pub(crate) struct Hitbox {
    pub(crate) mins: Vec2f,
    pub(crate) maxs: Vec2f,
}

impl Hitbox {
    pub(crate) fn corners(self, pos: Vec2f, angle: f64) -> [Vec2f; 4] {
        let back_left = pos + Vec2f::new(self.mins.x, self.mins.y).rotated_z(angle);
        let front_left = pos + Vec2f::new(self.maxs.x, self.mins.y).rotated_z(angle);
        let front_right = pos + Vec2f::new(self.maxs.x, self.maxs.y).rotated_z(angle);
        let back_right = pos + Vec2f::new(self.mins.x, self.maxs.y).rotated_z(angle);
        [back_left, front_left, front_right, back_right]
    }
}

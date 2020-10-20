//! The C in ECS
//!
//! Some structs/enums here are not components but their members.
//!
//! Some components have pure member functions.
//! This is not a violation of the ECS pattern,
//! because they don't modify game state - they're not behavior.

use enumn::N;
use legion::Entity;

use crate::{cvars::Cvars, map::Vec2f};

#[derive(Debug, Clone)]
pub(crate) struct Vehicle {
    pub(crate) veh_type: VehicleType,
    pub(crate) turret_angle: f64,
    /// HP between 0 and 1 - saving the fraction here so the cvars can be adjusted during a match.
    pub(crate) hp_fraction: f64,
    /// Each weapon has a separate reload status even if they all reload at the same time.
    /// I plan to generalize this and have a cvar to choose between multiple reload mechanisms.
    pub(crate) ammos: Vec<Ammo>,
    pub(crate) cur_weapon: Weapon,
}

impl Vehicle {
    #[must_use]
    pub(crate) fn new(cvars: &Cvars, veh_type: VehicleType) -> Vehicle {
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
            veh_type,
            turret_angle: 0.0,
            hp_fraction: 1.0,
            ammos,
            cur_weapon: Weapon::Mg,
        }
    }

    pub(crate) fn damage(&mut self, cvars: &Cvars, amount: f64) {
        self.hp_fraction -= amount / cvars.g_vehicle_hp(self.veh_type);
        if self.hp_fraction < 0.0 {
            self.hp_fraction = 0.0;
        }
    }

    pub(crate) fn destroyed(&self) -> bool {
        self.hp_fraction <= 0.0
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, N)]
pub(crate) enum VehicleType {
    Tank,
    Hovercraft,
    Hummer,
}

#[derive(Debug, Clone)]
pub(crate) enum Ammo {
    /// Refire delay end time, ammo count remaining
    Loaded(f64, u32),
    /// Start time, end time
    Reloading(f64, f64),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Pos(pub(crate) Vec2f);

#[derive(Debug, Clone, Copy)]
pub(crate) struct Vel(pub(crate) Vec2f);

#[derive(Debug, Clone, Copy)]
pub(crate) struct Angle(pub(crate) f64);

#[derive(Debug, Clone, Copy)]
pub(crate) struct TurnRate(pub(crate) f64);

#[derive(Debug, Clone, Copy)]
pub(crate) struct Time(pub(crate) f64);

pub(crate) const WEAPS_CNT: u8 = 7;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, N)]
pub(crate) enum Weapon {
    Mg,
    Rail,
    Cb,
    Rockets,
    Hm,
    Gm,
    Bfg,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Mg;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Cb;

#[derive(Debug, Clone, Copy)]
pub(crate) struct GuidedMissile;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Bfg;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Owner(pub(crate) Entity);

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

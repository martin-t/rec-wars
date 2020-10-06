//! Helpers for manipulating entities.

use legion::{query::IntoQuery, Entity, World};

use crate::map::Map;
use crate::{
    components::Angle, components::Hitbox, components::Pos, components::TurnRate,
    components::VehicleType, components::Vel, components::Weapon, cvars::Cvars,
};

#[derive(Debug, Clone)]
pub(crate) struct Vehicle {
    pub(crate) veh_type: VehicleType,
    pub(crate) destroyed: bool,
    pub(crate) turret_angle: f64,
    /// Fraction of full
    pub(crate) hp: f64,
    /// Each weapon has a separate reload status even if they all reload at the same time.
    /// I plan to generalize this and have a cvar to choose between multiple reload mechanisms.
    pub(crate) ammos: Vec<Ammo>,
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
            destroyed: false,
            turret_angle: 0.0,
            hp: 1.0,
            ammos,
        }
    }
}

pub(crate) fn all_vehicles(world: &World) -> Vec<(Entity, bool, Pos, Angle, Hitbox)> {
    let mut query_vehicles = <(Entity, &Vehicle, &Pos, &Angle, &Hitbox)>::query();
    query_vehicles
        .iter(world)
        .map(|(&entity, vehicle, &pos, &angle, &hitbox)| {
            (entity, vehicle.destroyed, pos, angle, hitbox)
        })
        .collect()
}

#[derive(Debug, Clone)]
pub(crate) enum Ammo {
    /// Refire delay end time, ammo count remaining
    Loaded(f64, u32),
    /// Start time, end time
    Reloading(f64, f64),
}

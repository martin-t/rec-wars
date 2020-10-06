//! Helpers for manipulating entities.

use legion::{query::IntoQuery, Entity, World};

use crate::components::{Angle, Hitbox, Pos, Vehicle};

pub(crate) fn all_vehicles(world: &World) -> Vec<(Entity, bool, Pos, Angle, Hitbox)> {
    let mut query_vehicles = <(Entity, &Vehicle, &Pos, &Angle, &Hitbox)>::query();
    query_vehicles
        .iter(world)
        .map(|(&entity, vehicle, &pos, &angle, &hitbox)| {
            (entity, vehicle.destroyed, pos, angle, hitbox)
        })
        .collect()
}

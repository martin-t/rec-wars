// LATER Move weapons code here.

use crate::prelude::*;

// LATER This is all wrong, should be on context, cl needs prev pos to find nearest point.
pub fn bfg_beam_hit(cvars: &Cvars, map: &Map, nearest_point: Vec2f, vehicle_pos: Vec2f) -> bool {
    let dist2 = (nearest_point - vehicle_pos).magnitude_squared();
    dist2 <= cvars.g_bfg_beam_range * cvars.g_bfg_beam_range
        && map.is_wall_trace(nearest_point, vehicle_pos).is_none()
}

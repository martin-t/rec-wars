use legion::{query::IntoQuery, World};

use crate::{
    components::Bfg, components::Pos, components::Vel, cvars::Cvars, game_state::Explosion,
    game_state::GameState, map::Map,
};

pub(crate) fn bfg(cvars: &Cvars, world: &mut World, map: &Map, gs: &mut GameState) {
    let mut to_remove = Vec::new();

    let mut query = <(legion::Entity, &Bfg, &mut Pos, &Vel)>::query();
    for (&entity, _, pos, vel) in query.iter_mut(world) {
        pos.0 += vel.0 * gs.dt;

        if map.collision(pos.0) {
            gs.explosions.push(Explosion::new(
                pos.0,
                cvars.g_bfg_explosion_scale,
                gs.frame_time,
                true,
            ));
            to_remove.push(entity);
        }
    }

    for entity in to_remove {
        world.remove(entity);
    }
}

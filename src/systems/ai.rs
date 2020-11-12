//! Stub. So far bots move and shoot randomly.

use legion::{EntityStore, IntoQuery, World};
use rand::Rng;

use crate::{
    components::{Ai, Player, Pos, Vehicle},
    game_state::{GameState, Input},
};

pub(crate) fn ai(world: &mut World, gs: &mut GameState) {
    let mut query_ai = <(&Player, &mut Ai, &mut Input)>::query();
    let (mut world_ai, world_rest) = world.split_for_query(&query_ai);
    for (player, ai, input) in query_ai.iter_mut(&mut world_ai) {
        let (spawn_time, pos) = if let Some(veh_entity) = player.vehicle {
            let veh_entry = world_rest.entry_ref(veh_entity).unwrap();
            let vehicle = veh_entry.get_component::<Vehicle>().unwrap();
            let pos = veh_entry.get_component::<Pos>().unwrap();
            (vehicle.spawn_time, *pos)
        } else {
            continue;
        };

        // keep moving forward if recently spawned
        let age = gs.frame_time - spawn_time;
        if age < 0.5 {
            ai.movement = 1;
        } else if gs.rng.gen_bool(0.01) {
            let r: f64 = gs.rng.gen();
            if r < 0.5 {
                ai.movement = 1;
            } else if r < 0.65 {
                ai.movement = 0;
            } else {
                ai.movement = -1;
            }
        }

        if gs.rng.gen_bool(0.03) {
            ai.turning = gs.rng.gen_range(-1, 2);
        }

        if !ai.firing && gs.rng.gen_bool(0.01) {
            ai.firing = true;
        } else if ai.firing && gs.rng.gen_bool(0.03) {
            ai.firing = false;
        }

        dbg_world_textf!(pos.0, "{:.1} {}", age, ai.movement);

        *input = Input {
            up: ai.movement == 1,
            down: ai.movement == -1,
            left: ai.turning == -1,
            right: ai.turning == 1,
            turret_left: gs.rng.gen_bool(0.3),
            turret_right: gs.rng.gen_bool(0.3),
            prev_weapon: gs.rng.gen_bool(0.02),
            next_weapon: gs.rng.gen_bool(0.01),
            fire: ai.firing,
            mine: gs.rng.gen_bool(0.001),
            self_destruct: gs.rng.gen_bool(0.0001),
            horn: gs.rng.gen_bool(0.0001),
            chat: false,
        }
    }
}

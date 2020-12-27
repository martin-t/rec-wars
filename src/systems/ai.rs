//! Stub. So far bots move and shoot randomly.

use rand::Rng;

use crate::{
    cvars::Cvars,
    game_state::{GameState, Input},
};

pub(crate) fn ai(cvars: &Cvars, gs: &mut GameState) {
    if !cvars.ai {
        return;
    }

    for (_, ai) in gs.ais.iter_mut() {
        let player = &mut gs.players[ai.player];
        let vehicle = &gs.vehicles[player.vehicle.unwrap()];

        // keep moving forward if recently spawned
        let age = gs.frame_time - vehicle.spawn_time;
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
            ai.turning = gs.rng.gen_range(-1..=1);
        }

        if !ai.firing && gs.rng.gen_bool(0.01) {
            ai.firing = true;
        } else if ai.firing && gs.rng.gen_bool(0.03) {
            ai.firing = false;
        }

        dbg_world_textf!(vehicle.pos, "{:.1} {}", age, ai.movement);

        player.input = Input {
            up: ai.movement == 1,
            down: ai.movement == -1,
            left: ai.turning == -1,
            right: ai.turning == 1,
            turret_left: gs.rng.gen_bool(0.01),
            turret_right: gs.rng.gen_bool(0.01),
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

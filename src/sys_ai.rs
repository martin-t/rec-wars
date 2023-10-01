//! Stub. So far bots move and shoot randomly.

use crate::prelude::*;

impl ServerFrameCtx<'_> {
    pub fn sys_ai(&mut self) {
        if !self.cvars.ai {
            return;
        }

        for (_, ai) in self.gs.ais.iter_mut() {
            let player = &mut self.gs.players[ai.player];
            let vehicle = &self.gs.vehicles[player.vehicle.unwrap()];

            // keep moving forward if recently spawned
            let age = self.gs.game_time - vehicle.spawn_time;
            if age < 0.5 {
                ai.movement = 1;
            } else if self.sg.rng.gen_bool(0.01) {
                let r: f64 = self.sg.rng.gen();
                if r < 0.5 {
                    ai.movement = 1;
                } else if r < 0.65 {
                    ai.movement = 0;
                } else {
                    ai.movement = -1;
                }
            }

            if self.sg.rng.gen_bool(0.03) {
                ai.turning = self.sg.rng.gen_range(-1..=1);
            }

            if !ai.firing && self.sg.rng.gen_bool(0.01) {
                ai.firing = true;
            } else if ai.firing && self.sg.rng.gen_bool(0.03) {
                ai.firing = false;
            }

            player.input = NetInput {
                up: ai.movement == 1,
                down: ai.movement == -1,
                left: ai.turning == -1,
                right: ai.turning == 1,
                turret_left: self.sg.rng.gen_bool(0.01),
                turret_right: self.sg.rng.gen_bool(0.01),
                prev_weapon: self.sg.rng.gen_bool(0.02),
                next_weapon: self.sg.rng.gen_bool(0.01),
                fire: ai.firing,
                mine: self.sg.rng.gen_bool(0.001),
                self_destruct: self.sg.rng.gen_bool(0.0001),
                horn: self.sg.rng.gen_bool(0.0001),
            }
        }
    }
}

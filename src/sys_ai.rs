//! Stub. So far bots move and shoot randomly.

use crate::prelude::*;

impl FrameCtx<'_> {
    pub fn ai(&mut self) {
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
            } else if self.gs.rng.gen_bool(0.01) {
                let r: f64 = self.gs.rng.gen();
                if r < 0.5 {
                    ai.movement = 1;
                } else if r < 0.65 {
                    ai.movement = 0;
                } else {
                    ai.movement = -1;
                }
            }

            if self.gs.rng.gen_bool(0.03) {
                ai.turning = self.gs.rng.gen_range(-1..=1);
            }

            if !ai.firing && self.gs.rng.gen_bool(0.01) {
                ai.firing = true;
            } else if ai.firing && self.gs.rng.gen_bool(0.03) {
                ai.firing = false;
            }

            player.input = Input {
                up: ai.movement == 1,
                down: ai.movement == -1,
                left: ai.turning == -1,
                right: ai.turning == 1,
                turret_left: self.gs.rng.gen_bool(0.01),
                turret_right: self.gs.rng.gen_bool(0.01),
                prev_weapon: self.gs.rng.gen_bool(0.02),
                next_weapon: self.gs.rng.gen_bool(0.01),
                fire: ai.firing,
                mine: self.gs.rng.gen_bool(0.001),
                self_destruct: self.gs.rng.gen_bool(0.0001),
                horn: self.gs.rng.gen_bool(0.0001),
                chat: false,
                pause: false, // :)
            }
        }
    }
}

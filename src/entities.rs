use rand::prelude::*;

use vek::Clamp;

use crate::cvars::Cvars;
use crate::{
    data::{Kind, Map, Surface, Vec2f},
    Input,
};

/// Returns (pos, angle).
pub fn random_spawn_pos(rng: &mut SmallRng, map: &Map) -> (Vec2f, f64) {
    let r = rng.gen_range(0, map.spawns().len());
    let index = map.spawns()[r];
    let pos = map.tile_center(index);
    let angle = map[index].angle;
    (pos, angle)
}

#[derive(Debug, Clone)]
pub struct GuidedMissile {
    pub pos: Vec2f,
    pub vel: Vec2f,
    /// Kinda like angular momentum, except more special-casey.
    /// TODO Might wanna revisit when i have proper physics.
    pub turn_rate: f64,
}

impl GuidedMissile {
    #[must_use]
    pub fn spawn(cvars: &Cvars, pos: Vec2f, angle: f64) -> GuidedMissile {
        // example of GM pasing through wall:
        // pos: Vec2f::new(640.0, 640.0),
        // vel: Vec2f::new(0.3, 0.2),

        GuidedMissile {
            pos,
            vel: Vec2f::new(cvars.g_guided_missile_speed_initial, 0.0).rotated_z(angle),
            turn_rate: 0.0,
        }
    }

    pub fn input(&mut self, dt: f64, cvars: &Cvars, input: &Input) {
        // Accel / decel
        let accel = input.up * cvars.g_guided_missile_speed_change * dt
            - input.down * cvars.g_guided_missile_speed_change * dt;
        let dir = self.vel.normalized();
        let speed_old = self.vel.magnitude();
        let speed_new = (speed_old + accel).clamped(
            cvars.g_guided_missile_speed_min,
            cvars.g_guided_missile_speed_max,
        );
        self.vel = speed_new * dir;

        // Turning
        // TODO this doesn't feel like flying a missile - probably needs to carry some sideways momentum
        let tr_input: f64 = input.right * cvars.g_guided_missile_turn_rate_increase * dt
            - input.left * cvars.g_guided_missile_turn_rate_increase * dt;

        // Without input, turn rate should gradually decrease towards 0.
        let tr_old = self.turn_rate;
        let tr = if tr_input == 0.0 {
            // With a fixed timestep, this would multiply tr_old each frame.
            let tr_after_friction = tr_old * cvars.g_guided_missile_turn_rate_friction.powf(dt);
            let exponential = (tr_old - tr_after_friction).abs();
            // With a fixed timestep, this would subtract from tr_old each frame.
            let linear = cvars.g_guided_missile_turn_rate_decrease * dt;
            // Don't auto-decay faster than turning in the other dir would.
            let max_change = cvars.g_guided_missile_turn_rate_increase * dt;
            let decrease = (exponential + linear).min(max_change);
            // Don't cross 0 and start turning in the other dir
            let tr_new = if tr_old > 0.0 {
                (tr_old - decrease).max(0.0)
            } else {
                (tr_old + decrease).min(0.0)
            };

            tr_new
        } else {
            (tr_old + tr_input).clamped(
                -cvars.g_guided_missile_turn_rate_max,
                cvars.g_guided_missile_turn_rate_max,
            )
        };

        self.vel.rotate_z(tr * dt);
        self.turn_rate = tr;
    }

    /// Returns if it hit something.
    pub fn physics(&mut self, dt: f64, map: &Map, surfaces: &Vec<Surface>) -> bool {
        // TODO this is broken when minimized (collision detection, etc.)
        self.pos += self.vel * dt;
        if self.pos.x <= 0.0 {
            return true;
        }
        if self.pos.y <= 0.0 {
            return true;
        }
        let map_size = map.maxs();
        if self.pos.x >= map_size.x {
            return true;
        }
        if self.pos.y >= map_size.y {
            return true;
        }

        let tile_pos = map.tile_pos(self.pos);
        let surface = map[tile_pos.index].surface;
        let kind = surfaces[surface].kind;
        if kind == Kind::Wall {
            return true;
        }

        false
    }
}

#[derive(Debug, Clone)]
pub struct Tank {
    pub pos: Vec2f,
    pub vel: Vec2f,
    pub angle: f64,
    pub angular_momentum: f64,
}

impl Tank {
    #[must_use]
    pub fn spawn(pos: Vec2f, angle: f64) -> Tank {
        Tank {
            pos,
            vel: Vec2f::zero(),
            angle,
            angular_momentum: 0.0,
        }
    }
}

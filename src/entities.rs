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

    pub fn physics(&mut self, dt: f64, cvars: &Cvars, input: &Input) {
        // Accel / decel
        let accel_input = input.up * cvars.g_guided_missile_speed_change
            - input.down * cvars.g_guided_missile_speed_change;
        let accel = accel_input * dt;
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
            let linear = (tr_old - tr_after_friction).abs();
            // With a fixed timestep, this would subtract from tr_old each frame.
            let constant = cvars.g_guided_missile_turn_rate_decrease * dt;
            // Don't auto-decay faster than turning in the other dir would.
            let max_change = cvars.g_guided_missile_turn_rate_increase * dt;
            let decrease = (linear + constant).min(max_change);
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
    pub fn collisions(&mut self, dt: f64, map: &Map, surfaces: &Vec<Surface>) -> bool {
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
    pub turn_rate: f64,
    pub charge: f64,
}

impl Tank {
    #[must_use]
    pub fn spawn(pos: Vec2f, angle: f64) -> Tank {
        Tank {
            pos,
            vel: Vec2f::zero(),
            angle,
            turn_rate: 0.0,
            charge: 1.0,
        }
    }

    pub fn tick(&mut self, dt: f64, cvars: &Cvars, input: &Input) {
        // Turn rate
        dbgf!("tank orig tr: {}", self.turn_rate);
        let tr_input = cvars.g_tank_turn_rate_increase * input.right
            - cvars.g_tank_turn_rate_increase * input.left;
        let tr_change = tr_input * dt;
        dbgd!(tr_change);
        self.turn_rate += tr_change;

        let tr_fric_const = cvars.g_tank_turn_rate_friction_const * dt;
        dbgd!(tr_fric_const);
        if self.turn_rate >= 0.0 {
            self.turn_rate = (self.turn_rate - tr_fric_const).max(0.0);
        } else {
            self.turn_rate = (self.turn_rate + tr_fric_const).min(0.0);
        }

        let tr_new = self.turn_rate * (1.0-cvars.g_tank_turn_rate_friction_linear).powf(dt);
        dbgf!("diff: {:?}", self.turn_rate - tr_new);
        self.turn_rate = tr_new.clamped(-cvars.g_tank_turn_rate_max, cvars.g_tank_turn_rate_max);
        dbgd!(self.turn_rate);

        // Accel / decel
        // TODO lateral friction
        dbgf!("tank orig speed: {}", self.vel.magnitude());
        let vel_input =
            cvars.g_tank_accel_forward * input.up - cvars.g_tank_accel_backward * input.down;
        let vel_change = vel_input * dt;
        dbgd!(vel_change);
        self.vel += Vec2f::unit_x().rotated_z(self.angle) * vel_change;

        let vel_fric_const = cvars.g_tank_friction_const * dt;
        dbgd!(vel_fric_const);
        let vel_norm = self.vel.try_normalized().unwrap_or_default();
        self.vel -= (vel_fric_const).min(self.vel.magnitude()) * vel_norm;

        let vel_new = self.vel * cvars.g_tank_friction_linear.powf(dt);
        dbgf!("diff: {:?}", (self.vel - vel_new).magnitude());
        self.vel = vel_new;
        if self.vel.magnitude_squared() > cvars.g_tank_speed_max.powi(2) {
            self.vel = vel_norm * cvars.g_tank_speed_max;
        }
        dbgd!(self.vel.magnitude());

        // Turning - part of vel gets rotated
        let vel_rotation = self.turn_rate * cvars.g_tank_turn_effectiveness;
        self.vel.rotate_z(vel_rotation);
        self.angle += self.turn_rate;

        // TODO unify order with missile / input

        // Moving
        self.pos += self.vel * dt;

        // Reloading
        self.charge = (self.charge + dt).min(1.0);
    }
}

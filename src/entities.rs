use legion::{query::IntoQuery, Entity, World};
use vek::Clamp;

use crate::{
    components::Angle, components::Hitbox, components::Pos, components::TurnRate, components::Vel,
    cvars::Cvars, weapons::Weapon,
};
use crate::{
    map::{Map, Vec2f},
    Input,
};

#[derive(Debug, Clone)]
pub(crate) struct GuidedMissile {
    pub(crate) pos: Vec2f,
    pub(crate) vel: Vec2f,
    /// Kinda like angular momentum, except more special-casey.
    /// TODO Might wanna revisit when i have proper physics.
    pub(crate) turn_rate: f64,
}

impl GuidedMissile {
    #[must_use]
    pub(crate) fn spawn(cvars: &Cvars, pos: Vec2f, angle: f64) -> GuidedMissile {
        // example of GM pasing through wall:
        // pos: Vec2f::new(640.0, 640.0),
        // vel: Vec2f::new(0.3, 0.2),

        GuidedMissile {
            pos,
            vel: Vec2f::new(cvars.g_guided_missile_speed_initial, 0.0).rotated_z(angle),
            turn_rate: 0.0,
        }
    }

    /// Returns if it hit something.
    pub(crate) fn tick(&mut self, dt: f64, cvars: &Cvars, input: &Input, map: &Map) -> bool {
        // Accel / decel
        let accel_input = input.up_down() * cvars.g_guided_missile_speed_change * dt;
        let dir = self.vel.normalized();
        let speed_old = self.vel.magnitude();
        let speed_new = (speed_old + accel_input).clamped(
            cvars.g_guided_missile_speed_min,
            cvars.g_guided_missile_speed_max,
        );
        self.vel = speed_new * dir;

        // Turning
        // TODO this doesn't feel like flying a missile - probably needs to carry some sideways momentum
        let tr_input: f64 = input.right_left() * cvars.g_guided_missile_turn_rate_increase * dt;

        // Without input, turn rate should gradually decrease towards 0.
        let tr_old = self.turn_rate;
        let tr = if tr_input == 0.0 {
            // With a fixed timestep, this would multiply tr_old each frame.
            let tr_after_friction =
                tr_old * (1.0 - cvars.g_guided_missile_turn_rate_friction).powf(dt);
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

        // TODO this is broken when minimized (collision detection, etc.)
        self.pos += self.vel * dt;
        map.collision(self.pos)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PlayerVehicle {
    pub(crate) turret_angle: f64,
    /// Fraction of full
    pub(crate) hp: f64,
    /// Each weapon has a separate reload status even if they all reload at the same time.
    /// I plan to generalize this and have a cvar to choose between multiple reload mechanisms.
    pub(crate) ammos: Vec<Ammo>,
}

impl PlayerVehicle {
    #[must_use]
    pub(crate) fn new(cvars: &Cvars) -> PlayerVehicle {
        let ammos = vec![
            Ammo::Loaded(0.0, cvars.g_weapon_reload_ammo(Weapon::Mg)),
            Ammo::Loaded(0.0, cvars.g_weapon_reload_ammo(Weapon::Rail)),
            Ammo::Loaded(0.0, cvars.g_weapon_reload_ammo(Weapon::Cb)),
            Ammo::Loaded(0.0, cvars.g_weapon_reload_ammo(Weapon::Rockets)),
            Ammo::Loaded(0.0, cvars.g_weapon_reload_ammo(Weapon::Hm)),
            Ammo::Loaded(0.0, cvars.g_weapon_reload_ammo(Weapon::Gm)),
            Ammo::Loaded(0.0, cvars.g_weapon_reload_ammo(Weapon::Bfg)),
        ];

        PlayerVehicle {
            turret_angle: 0.0,
            hp: 1.0,
            ammos,
        }
    }

    pub(crate) fn tick(
        &mut self,
        dt: f64,
        cvars: &Cvars,
        input: &Input,
        map: &Map,
        pos: &mut Pos,
        vel: &mut Vel,
        angle: &mut Angle,
        turn_rate: &mut TurnRate,
        hitbox: &Hitbox,
    ) {
        // Turn rate
        dbg_textf!("tank orig tr: {}", turn_rate.0);
        let tr_change = input.right_left() * cvars.g_tank_turn_rate_increase * dt;
        dbg_textd!(tr_change);
        turn_rate.0 += tr_change;

        let tr_fric_const = cvars.g_tank_turn_rate_friction_const * dt;
        dbg_textd!(tr_fric_const);
        if turn_rate.0 >= 0.0 {
            turn_rate.0 = (turn_rate.0 - tr_fric_const).max(0.0);
        } else {
            turn_rate.0 = (turn_rate.0 + tr_fric_const).min(0.0);
        }

        let tr_new = turn_rate.0 * (1.0 - cvars.g_tank_turn_rate_friction_linear).powf(dt);
        dbg_textf!("diff: {:?}", turn_rate.0 - tr_new);
        turn_rate.0 = tr_new.clamped(-cvars.g_tank_turn_rate_max, cvars.g_tank_turn_rate_max);
        dbg_textd!(turn_rate.0);

        // Accel / decel
        // TODO lateral friction
        dbg_textf!("tank orig speed: {}", vel.0.magnitude());
        let vel_change = input.up_down() * cvars.g_tank_accel_forward * dt;
        dbg_textd!(vel_change);
        vel.0 += Vec2f::unit_x().rotated_z(angle.0) * vel_change;

        let vel_fric_const = cvars.g_tank_friction_const * dt;
        dbg_textd!(vel_fric_const);
        let vel_norm = vel.0.try_normalized().unwrap_or_default();
        vel.0 -= (vel_fric_const).min(vel.0.magnitude()) * vel_norm;

        let vel_new = vel.0 * (1.0 - cvars.g_tank_friction_linear).powf(dt);
        dbg_textf!("diff: {:?}", (vel.0 - vel_new).magnitude());
        vel.0 = vel_new;
        if vel.0.magnitude_squared() > cvars.g_tank_speed_max.powi(2) {
            vel.0 = vel_norm * cvars.g_tank_speed_max;
        }
        dbg_textd!(vel.0.magnitude());

        // Turning - part of vel gets rotated to simulate steering
        // TODO cvar to set turning origin - original RW turned around turret center
        let vel_rotation = turn_rate.0 * cvars.g_tank_turn_effectiveness;
        vel.0.rotate_z(vel_rotation);
        let new_angle = angle.0 + turn_rate.0; // TODO * dt
        if corners(*hitbox, pos.0, new_angle)
            .iter()
            .any(|&corner| map.collision(corner))
        {
            turn_rate.0 *= -0.5;
        } else {
            angle.0 = new_angle;
        }

        // TODO unify order with missile / input

        // Moving
        let new_pos = pos.0 + vel.0 * dt;
        if corners(*hitbox, new_pos, angle.0)
            .iter()
            .any(|&corner| map.collision(corner))
        {
            vel.0 *= -0.5;
        } else {
            pos.0 = new_pos;
        }
    }
}

pub(crate) fn corners(hitbox: Hitbox, pos: Vec2f, angle: f64) -> [Vec2f; 4] {
    let back_left = pos + Vec2f::new(hitbox.mins.x, hitbox.mins.y).rotated_z(angle);
    let front_left = pos + Vec2f::new(hitbox.maxs.x, hitbox.mins.y).rotated_z(angle);
    let front_right = pos + Vec2f::new(hitbox.maxs.x, hitbox.maxs.y).rotated_z(angle);
    let back_right = pos + Vec2f::new(hitbox.mins.x, hitbox.maxs.y).rotated_z(angle);
    [back_left, front_left, front_right, back_right]
}

pub(crate) fn all_vehicles(world: &World) -> Vec<(Entity, Pos, Angle, Hitbox)> {
    let mut query_vehicles = <(Entity, &Pos, &Angle, &Hitbox)>::query();
    query_vehicles
        .iter(world)
        .map(|(&entity, &pos, &angle, &hitbox)| (entity, pos, angle, hitbox))
        .collect()
}

#[derive(Debug, Clone)]
pub(crate) enum Ammo {
    /// Refire delay end time, ammo count remaining
    Loaded(f64, u32),
    /// Start time, end time
    Reloading(f64, f64),
}

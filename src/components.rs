//! The C in ECS

use enumn::N;
use legion::Entity;

use crate::map::Vec2f;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Pos(pub(crate) Vec2f);

#[derive(Debug, Clone, Copy)]
pub(crate) struct Vel(pub(crate) Vec2f);

#[derive(Debug, Clone, Copy)]
pub(crate) struct Angle(pub(crate) f64);

#[derive(Debug, Clone, Copy)]
pub(crate) struct TurnRate(pub(crate) f64);

#[derive(Debug, Clone, Copy)]
pub(crate) struct Time(pub(crate) f64);

pub(crate) const WEAPS_CNT: u8 = 7;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, N)]
pub(crate) enum Weapon {
    Mg,
    Rail,
    Cb,
    Rockets,
    Hm,
    Gm,
    Bfg,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Owner(pub(crate) Entity);

#[derive(Debug, Clone, Copy)]
pub(crate) struct Hitbox {
    pub(crate) mins: Vec2f,
    pub(crate) maxs: Vec2f,
}

impl Hitbox {
    pub(crate) fn corners(self, pos: Vec2f, angle: f64) -> [Vec2f; 4] {
        let back_left = pos + Vec2f::new(self.mins.x, self.mins.y).rotated_z(angle);
        let front_left = pos + Vec2f::new(self.maxs.x, self.mins.y).rotated_z(angle);
        let front_right = pos + Vec2f::new(self.maxs.x, self.maxs.y).rotated_z(angle);
        let back_right = pos + Vec2f::new(self.mins.x, self.maxs.y).rotated_z(angle);
        [back_left, front_left, front_right, back_right]
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, N)]
pub(crate) enum VehicleType {
    Tank,
    Hovercraft,
    Hummer,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Destroyed(pub(crate) bool);

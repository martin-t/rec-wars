use crate::map::Vec2f;

#[derive(Debug, Clone, Copy)]
pub struct Pos(pub Vec2f);

#[derive(Debug, Clone, Copy)]
pub struct Vel(pub Vec2f);

#[derive(Debug, Clone, Copy)]
pub struct Angle(pub f64);

#[derive(Debug, Clone, Copy)]
pub struct Time(pub f64);

#[derive(Debug, Clone, Copy)]
pub struct Cb;

#[derive(Debug, Clone, Copy)]
pub struct Rocket;

#[derive(Debug, Clone, Copy)]
pub struct Hitbox {
    pub mins: Vec2f,
    pub maxs: Vec2f,
}

impl Hitbox {
    pub fn new(mins: Vec2f, maxs: Vec2f) -> Self {
        Self { mins, maxs }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum Vehicle {
    Tank,
    Hovercraft,
    Hummer,
}

use crate::map::Vec2f;

#[derive(Debug)]
pub struct Pos(pub Vec2f);

#[derive(Debug)]
pub struct Vel(pub Vec2f);

#[derive(Debug)]
pub struct Angle(pub f64);

#[derive(Debug)]
pub struct Hitbox {
    pub mins: Vec2f,
    pub maxs: Vec2f,
}

impl Hitbox {
    pub fn new(mins: Vec2f, maxs: Vec2f) -> Self {
        Self { mins, maxs }
    }
}

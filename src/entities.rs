use crate::data::{Map, Vec2f};

#[derive(Debug, Clone)]
pub struct GuidedMissile {
    pub pos: Vec2f,
    pub vel: Vec2f,
    pub dir: Vec2f,
    pub speed: f64,
    pub turn_rate: f64,
}

pub fn spawn_guided_missile(map: &Map) -> GuidedMissile {
    // example of GM pasing through wall:
    // pos: Vec2f::new(640.0, 640.0),
    // vel: Vec2f::new(0.3, 0.2),

    //self.pos = Vec2f::new(640.0, 640.0);

    GuidedMissile {
        pos: Vec2f::new(640.0, 640.0),
        vel: Vec2f::new(0.3, 0.2),
        dir: Vec2f::new(0.3, 0.2),
        speed: 50.0,
        turn_rate: 0.0,
    }
}

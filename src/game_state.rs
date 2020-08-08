use crate::data::Vec2f;
use crate::entities::GuidedMissile;

#[derive(Debug, Clone)]
pub struct GameState {
    pub guided_missile: GuidedMissile,
    pub explosions: Vec<(Vec2f, i32)>,
}

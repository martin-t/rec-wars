use crate::data::Vec2f;
use crate::entities::GuidedMissile;

/// Everyting that changes during the game
/// and might need to be taken back during frame interpolation / reconciliation.
#[derive(Debug, Clone)]
pub struct GameState {
    pub guided_missile: GuidedMissile,
    pub explosions: Vec<(Vec2f, i32)>,
}

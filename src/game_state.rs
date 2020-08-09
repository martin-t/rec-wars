use crate::data::Vec2f;
use crate::entities::{GuidedMissile, Tank};

/// Everyting that changes during the game
/// and might need to be taken back during frame interpolation / reconciliation.
#[derive(Debug, Clone)]
pub struct GameState {
    pub gm: GuidedMissile,
    pub tank: Tank,
    pub pe: PlayerEntity,
    pub explosions: Vec<(Vec2f, i32)>,
}

#[derive(Debug, Clone)]
pub enum PlayerEntity {
    GuidedMissile,
    Tank,
}

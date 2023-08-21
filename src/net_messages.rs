//! Messages sent between the client and server, usually over the network.
//!
//! LATER These will form the basis of demo recording and replay.

use serde::{Deserialize, Serialize};

use crate::{
    debug::details::{DebugShape, WorldText},
    prelude::*,
};

#[derive(Debug, Deserialize, Serialize)]
pub enum ClientMessage {
    Input(Input),
    Chat(String), // LATER Allow sending this
    Join,
    Observe,
}

// LATER Since messages get serialized immediately, consider using slices instead of Vecs to avoid allocations.

/// Message sent from server to client
#[derive(Debug, Deserialize, Serialize)]
pub enum ServerMessage {
    /// Initial game state that is sent to a new player upon connecting.
    ///
    /// This is intentionally separate from messages such as AddPlayer or SpawnVehicle
    /// because eventually those might trigger additional effects
    /// such as info messages, sounds, particles, etc.
    Init(Init),
    // TODO cleanup
    // /// Add a new player to the game.
    // AddPlayer(AddPlayer),
    // /// Remove the player and all data associated with him, for example when he disconnects.
    // RemovePlayer { player_index: u32 },
    // /// This player is now observing.
    // Observe { player_index: u32 },
    // /// This player is now spectating.
    // Spectate {
    //     player_index: u32,
    //     spectatee_index: u32,
    // },
    // /// This player is now playing.
    // Join { player_index: u32 },
    // /// Spawn a new cycle for an existing player.
    // SpawnCycle(PlayerCycle),
    // /// Remove the cycle from game state, for example when the player switches to observer mode.
    // DespawnCycle { cycle_index: u32 },
    /// Update the translations, rotations, velocities, etc. of everything.
    Update(Update),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Init {
    pub player_indices: Vec<u32>,
    pub local_player_index: u32,
    pub player_vehicles: Vec<(u32, u32)>,
    pub player_projectiles: Vec<(u32, u32)>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Update {
    pub player_inputs: Vec<(u32, Input)>, // TOOD(security) Whitelist (e.g. don't send chat PMs)
    pub vehicle_physics: Vec<(u32, EntityPhysics)>,
    pub debug_texts: Vec<String>,
    pub debug_texts_world: Vec<WorldText>,
    pub debug_shapes: Vec<DebugShape>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EntityPhysics {
    pub pos: Vec2f,
    pub vel: Vec2f,
    pub angle: f64,
    pub turn_rate: f64,
}

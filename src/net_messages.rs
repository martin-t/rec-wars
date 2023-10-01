//! Messages sent between the client and server, usually over the network.
//!
//! You might have noticed a weird trend.
//! Values of thunderdome's Index type are called handle and its u32 slots are called index.
//! That's to be consistent with how RustCycles does it.
//! Eventually we might switch to fyrox's Pool gen arena anyway.
//!
//! LATER These will form the basis of demo recording and replay.

use crate::{
    common::ServerTimings,
    debug::details::{DebugShape, WorldText},
    prelude::*,
};

#[derive(Debug, Deserialize, Serialize)]
pub enum ClientMessage {
    Connect(Connect),
    Input(NetInput),
    Chat(String), // LATER Allow sending this
    Pause,
    Join,
    Observe,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Connect {
    pub cl_version: String,
    pub name1: String,
    pub name2: Option<String>,
}

// #[derive(Debug, Deserialize, Serialize)]
// pub struct NetInput {
//     pub state: InputState,
//     pub events: Vec<InputEvent>,
// }

// LATER Since messages get serialized immediately, consider using slices instead of Vecs to avoid allocations.

/// Message sent from server to client
///
/// Naming:
///
/// - add/remove - bookkeeping events - player disconnects, visibility culling, ...
/// - spawn/destroy - game events with associated effects like explosions
///
/// At least that's the theory, turns out in RecWars spawning vehicles or projectiles doesn't have any effects.
///
/// The recommended usage when receiving is to destructure the data so you notice when new fields are added.
#[derive(Debug, Deserialize, Serialize)]
pub enum ServerMessage {
    /// Initial game state that is sent to a new player upon connecting.
    ///
    /// This is intentionally separate from messages such as AddPlayer or SpawnVehicle
    /// because eventually those might trigger additional effects
    /// such as info messages, sounds, particles, etc.
    Init(Init),

    /// Update the game state on all clients. Sent every server frame.
    Update(Update),

    /// Pause state changed.
    Paused(bool),

    AddPlayer(PlayerInit),
    SpawnVehicle(VehicleInit),
    SpawnProjectile(ProjectileInit),
    SpawnExplosion(ExplosionInit),

    RailBeam(RailBeam),

    /// Remove the player and all data associated with him, for example when he disconnects.
    RemovePlayer {
        index: u32,
    },
    /// Remove the projectile and create the associated effects (explosions, sounds, ...).
    DestroyProjectile {
        index: u32,
    },

    Kill(Kill),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Init {
    pub sv_version: String,
    pub map_path: String,
    pub frame_num: usize,
    pub game_time: f64,
    pub game_time_prev: f64,
    pub dt: f64,
    pub players: Vec<PlayerInit>,
    pub local_player1_index: u32,
    pub local_player2_index: Option<u32>,
    pub vehicles: Vec<VehicleInit>,
    pub projectiles: Vec<ProjectileInit>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerInit {
    pub index: u32,
    pub name: String,
    pub score: Score,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VehicleInit {
    pub index: u32,
    pub physics: EntityPhysics,
    pub veh_type: VehicleType,
    pub turret_angle_current: f64,
    pub turret_angle_wanted: f64,
    pub spawn_time: f64,
    pub owner: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectileInit {
    pub index: u32,
    pub weapon: Weapon,
    pub physics: EntityPhysics,
    pub explode_time: f64,
    pub owner: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExplosionInit {
    pub pos: Vec2f,
    pub scale: f64,
    pub bfg: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Update {
    pub frame_num: usize,
    pub game_time: f64,
    pub game_time_prev: f64,
    pub dt: f64,
    pub player_inputs: Vec<InputUpdate>,
    pub vehicles: Vec<VehicleUpdate>,
    pub projectiles: Vec<ProjectileUpdate>,
    pub debug_texts: Vec<String>,
    pub debug_texts_world: Vec<WorldText>,
    pub debug_shapes: Vec<DebugShape>,
    pub server_timings: ServerTimings,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InputUpdate {
    pub index: u32,
    pub net_input: NetInput,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VehicleUpdate {
    pub index: u32,
    pub physics: EntityPhysics,
    pub turret_angle_current: f64,
    pub turret_angle_wanted: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectileUpdate {
    pub index: u32,
    pub physics: EntityPhysics,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Kill {
    pub attacker: u32,
    pub victim: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EntityPhysics {
    pub pos: Vec2f,
    pub vel: Vec2f,
    pub angle: f64,
    pub turn_rate: f64,
}

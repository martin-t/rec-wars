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
    Version(Version),
    Connect(Connect),
    Input(NetInput),
    Chat(String), // LATER Allow sending this
    Pause,
    Join,
    Observe,
}

/// Description of the client or server version to determine compatibility.
///
/// This struct must remain stable across all versions
/// so old versions can parse the message from new versions.
#[derive(Debug, Deserialize, Serialize)]
pub struct Version {
    /// The name of the game, for example "RecWars" or "RustCycles".
    /// Since they use very similar protocols, this is used to make sure
    /// we're not accidentally connecting to the wrong game's server
    /// to save headaches debugging. Yes, this happened.
    pub game: String,

    pub major: u32,
    pub minor: u32,
    pub patch: u32,

    /// Pre-release identifier sometimes used in SemVer.
    /// For example "alpha.1" or "rc.0".
    pub pre: Option<String>,

    /// Number of commits since the last tag.
    pub commits: Option<u32>,

    /// Git commit hash.
    pub hash: Option<String>,

    /// Whether the working directory was dirty when building.
    pub dirty: Option<bool>,

    /// Any extra unstructured information.
    pub extra: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Connect {
    pub cl_version: String, // TODO remove
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
    Version(Version),

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

#[cfg(test)]
mod tests {
    use crate::net;

    use super::*;

    #[test]
    fn handshake_version_format() {
        // Chech this one message always has the same binary format
        // between different versions of the game.

        // Note that if we need to break compatibility
        // (e.g. by not using bincode),
        // there are still options left open.
        // The first 4 bytes (length) will never by less than 4
        // and the first message should be short anyway,
        // so we can always use a very low or high number
        // to indicate a different format.

        let v1 = Version {
            game: "RecWars".to_owned(),
            major: 0x45,
            minor: 0x1a4,
            patch: 0x539,
            pre: None,
            commits: None,
            hash: None,
            dirty: None,
            extra: None,
        };
        let v2 = Version {
            game: "RustCycles".to_owned(),
            major: 0x55555555,
            minor: 0xaaaaaaaa,
            patch: 0xffffffff,
            pre: Some("alpha.1".to_owned()),
            commits: Some(0x29a),
            hash: Some("deadbeef".to_owned()),
            dirty: Some(true),
            extra: Some(
                "MOTD: I learned a lot from my mistakes so i decided to make more mistakes to learn more.".to_owned(),
            ),
        };
        let msg1 = ClientMessage::Version(v1);
        let msg2 = ClientMessage::Version(v2);
        let serialized1 = net::serialize(msg1);
        let serialized2 = net::serialize(msg2);

        for b in &serialized1.bytes {
            print!("{:02x} ", b);
        }
        println!();

        for b in &serialized2.bytes {
            print!("{:02x} ", b);
        }
        println!();

        assert_eq!(
            serialized1.bytes,
            [
                0x28, 0x00, 0x00, 0x00, // total len
                0x00, 0x00, 0x00, 0x00, // message variant
                0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // name len
                0x52, 0x65, 0x63, 0x57, 0x61, 0x72, 0x73, // "RecWars"
                0x45, 0x00, 0x00, 0x00, // major
                0xa4, 0x01, 0x00, 0x00, // minor
                0x39, 0x05, 0x00, 0x00, // patch
                0x00, // pre - None
                0x00, // commits - None
                0x00, // hash - None
                0x00, // dirty - None
                0x00, // extra - None
            ]
        );
        assert_eq!(
            serialized2.bytes,
            [
                0xaf, 0x00, 0x00, 0x00, // total len
                0x00, 0x00, 0x00, 0x00, // message variant
                0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // name len
                0x52, 0x75, 0x73, 0x74, 0x43, 0x79, 0x63, 0x6c, 0x65, 0x73, // "RustCycles"
                0x55, 0x55, 0x55, 0x55, // major
                0xaa, 0xaa, 0xaa, 0xaa, // minor
                0xff, 0xff, 0xff, 0xff, // patch
                0x01, // pre - Some
                0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // pre len
                0x61, 0x6c, 0x70, 0x68, 0x61, 0x2e, 0x31, // "alpha.1"
                0x01, // commits - Some
                0x9a, 0x02, 0x00, 0x00, // commits
                0x01, // hash - Some
                0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // hash len
                0x64, 0x65, 0x61, 0x64, 0x62, 0x65, 0x65, 0x66, // "deadbeef"
                0x01, // dirty - Some
                0x01, // dirty
                0x01, // extra - Some
                0x58, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // extra len
                0x4d, 0x4f, 0x54, 0x44, 0x3a, 0x20, 0x49, 0x20, 0x6c, 0x65, 0x61, 0x72, 0x6e, 0x65,
                0x64, 0x20, 0x61, 0x20, 0x6c, 0x6f, 0x74, 0x20, 0x66, 0x72, 0x6f, 0x6d, 0x20, 0x6d,
                0x79, 0x20, 0x6d, 0x69, 0x73, 0x74, 0x61, 0x6b, 0x65, 0x73, 0x20, 0x73, 0x6f, 0x20,
                0x69, 0x20, 0x64, 0x65, 0x63, 0x69, 0x64, 0x65, 0x64, 0x20, 0x74, 0x6f, 0x20, 0x6d,
                0x61, 0x6b, 0x65, 0x20, 0x6d, 0x6f, 0x72, 0x65, 0x20, 0x6d, 0x69, 0x73, 0x74, 0x61,
                0x6b, 0x65, 0x73, 0x20, 0x74, 0x6f, 0x20, 0x6c, 0x65, 0x61, 0x72, 0x6e, 0x20, 0x6d,
                0x6f, 0x72, 0x65, 0x2e,
            ]
        )
    }
}

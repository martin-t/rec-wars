//! The authoritative server in a client-server game architecture
//!
//! All data affecting gameplay, players, bots, networking...

use std::{io::ErrorKind, mem, net::TcpListener};

use crate::{
    common::ServerTimings,
    debug::{self, DEBUG_SHAPES, DEBUG_TEXTS, DEBUG_TEXTS_WORLD},
    net::{self, Connection, Listener, NetworkMessage},
    prelude::*,
    timing::{Durations, Fps},
    BOT_NAMES,
};

pub struct Server {
    pub map: Map,

    pub gs: GameState,

    pub sg: ServerGame,

    /// Game time left over from previous update.
    pub dt_carry: f64,

    /// Time since the process started in seconds. Increases at wall clock speed even when paused.
    ///
    /// This is not meant to be used for anything that affects gameplay - use `gs.game_time` instead.
    pub real_time: f64,
    pub real_time_prev: f64,
    pub real_time_delta: f64,
}

pub struct ServerGame {
    pub listener: Box<dyn Listener<ClientMessage>>,
    pub clients: Arena<RemoteClient>,
    /// Handles to remote clients that have disconnected.
    pub disconnected: FnvHashSet<Index>,

    pub paused: bool,

    /// The RNG for all gamelogic
    pub rng: Xoshiro256PlusPlus,

    pub update_fps: Fps,
    pub update_durations: Durations,
    pub gamelogic_fps: Fps,
    pub gamelogic_durations: Durations,
}

#[derive(Debug)]
pub enum SendDest {
    /// Index to RemoteClient
    One(Index),
    All,
}

pub struct RemoteClient {
    conn: Box<dyn Connection<ClientMessage>>,
    player_handle: Index,
}

impl RemoteClient {
    fn new(conn: Box<dyn Connection<ClientMessage>>, player_handle: Index) -> Self {
        Self {
            conn,
            player_handle,
        }
    }
}

impl Server {
    pub fn new(cvars: &Cvars, map: Map) -> Self {
        let listener = TcpListener::bind(&cvars.sv_net_listen_addr).unwrap();
        listener.set_nonblocking(true).unwrap();
        dbg_logf!("Listening on {}", &cvars.sv_net_listen_addr);

        let sg = ServerGame {
            listener: Box::new(listener),
            clients: Arena::new(),
            disconnected: FnvHashSet::default(),

            paused: false,

            rng: Xoshiro256PlusPlus::seed_from_u64(cvars.d_seed),

            update_fps: Fps::new(),
            update_durations: Durations::new(),
            gamelogic_fps: Fps::new(),
            gamelogic_durations: Durations::new(),
        };

        Self {
            map,
            gs: GameState::new(),
            sg,

            dt_carry: 0.0,
            real_time: 0.0,
            real_time_prev: 0.0,
            real_time_delta: 0.0,
        }
    }

    pub fn ctx<'a>(&'a mut self, cvars: &'a Cvars) -> ServerFrameCtx<'a> {
        ServerFrameCtx::new(cvars, &self.map, &mut self.gs, &mut self.sg)
    }

    /// Run gamelogic frame(s) up to current time (in seconds).
    pub fn update(&mut self, cvars: &Cvars, real_time: f64) {
        // Recommended reading:
        // https://gafferongames.com/post/fix_your_timestep/
        // https://medium.com/@tglaiel/how-to-make-your-game-run-at-60fps-24c61210fe75

        self.sg.update_fps.tick(cvars.d_fps_period, self.real_time);
        let start = macroquad::time::get_time();

        // Update time tracking variables
        self.real_time_prev = self.real_time;
        self.real_time = real_time;
        self.real_time_delta = self.real_time - self.real_time_prev;

        // We have to also receive outside gamelogic so pausing and unpausing works.
        self.ctx(cvars).sys_net_receive(); // LATER Just receive, handle pause explicitly

        // LATER Remove explicit condition, just don't update time?
        //  Some systems should run even when paused (e.g. receive)? Move them from tick to update?
        if !self.sg.paused {
            let dt_update = self.real_time_delta * cvars.d_speed;
            self.gamelogic(cvars, dt_update);
        }

        let end = macroquad::time::get_time();
        self.sg
            .update_durations
            .add(cvars.d_timing_samples, end - start);
    }

    /// The main game loop.
    fn gamelogic(&mut self, cvars: &Cvars, dt_update: f64) {
        // LATER Slow down time to prevent death spirals.
        // LATER Extrapolation (after client / server split).
        //  Gamecode should not know about it.
        //  Construct FrameData with the throwaway gs, gamelogic_tick_movement that only calls the movement systems?
        //  Don't accidentally call functions which modify state outside gs.

        if dt_update > 5.0 {
            dbg_logf!("WARNING: large dt_update: {dt_update}");
        }

        match cvars.sys_tickrate_mode {
            TickrateMode::Variable => {
                let game_time_target = self.gs.game_time + dt_update;
                self.gamelogic_tick(cvars, game_time_target);
            }
            TickrateMode::Fixed => {
                let dt = 1.0 / cvars.sys_tickrate_fixed_fps;
                let game_time_target = self.gs.game_time + self.dt_carry + dt_update;

                while self.gs.game_time + dt < game_time_target {
                    self.gamelogic_tick(cvars, self.gs.game_time + dt);
                }

                self.dt_carry = game_time_target - self.gs.game_time;
                if cvars.d_tickrate_fixed_carry {
                    dbg_logf!("Remaining time: {}", self.dt_carry);
                }
            }
        }
    }

    /// Run one frame of gamelogic.
    fn gamelogic_tick(&mut self, cvars: &Cvars, game_time: f64) {
        let start = macroquad::time::get_time();
        self.sg
            .gamelogic_fps
            .tick(cvars.d_fps_period, self.real_time);

        // Update time tracking variables (in seconds)
        assert!(
            game_time >= self.gs.game_time,
            "game_time didn't increase: prev {}, current {}",
            self.gs.game_time,
            game_time,
        );
        self.gs.frame_num += 1;
        self.gs.game_time_prev = self.gs.game_time;
        self.gs.game_time = game_time;
        self.gs.dt = self.gs.game_time - self.gs.game_time_prev;
        debug::set_game_time(self.gs.game_time);

        debug::clear_expired();

        dbg_textf!("{}", env!("GIT_VERSION"));
        dbg_textd!(self.gs.game_time);
        dbg_textd!(self.gs.game_time_prev);

        let mut ctx = ServerFrameCtx::new(cvars, &self.map, &mut self.gs, &mut self.sg);

        ctx.sys_cleanup();

        ctx.sys_net_accept();
        ctx.sys_connect_bots();
        ctx.sys_net_receive();
        ctx.sys_net_disconnect();
        ctx.sys_ai();

        ctx.sys_respawning();

        ctx.sys_player_weapon();

        ctx.sys_vehicle_logic();

        // It's probably a good idea to shoot before movement so that when turning
        // the shot angle corresponds to the vehicle angle the player saw last frame.
        // LATER Before turret turning too.
        ctx.sys_shooting();

        ctx.sys_vehicle_movement();

        ctx.sys_hm_turning();
        ctx.sys_gm_turning();

        ctx.sys_projectiles();

        ctx.sys_projectiles_timeout();

        ctx.self_destruct();

        ctx.sys_debug_examples(v!(125, 300));

        ctx.sys_net_send_updates();
        ctx.sys_net_disconnect();

        dbg_textf!("vehicle count: {}", self.gs.vehicles.len());
        dbg_textf!("projectile count: {}", self.gs.projectiles.len());

        // LATER Remove prev, use state+events
        for (_, player) in self.gs.players.iter_mut() {
            player.input_prev = player.input;
        }

        let end = macroquad::time::get_time();
        self.sg
            .gamelogic_durations
            .add(cvars.d_timing_samples, end - start);
    }
}

impl ServerFrameCtx<'_> {
    // LATER not pub? only send in one place, instead most places record journal/replay/demo?
    pub fn net_send_all(&mut self, msg: ServerMessage) {
        let net_msg = net::serialize(msg);
        for (client_handle, client) in self.sg.clients.iter_mut() {
            Self::net_send(
                &net_msg,
                client_handle,
                client,
                self.gs,
                &mut self.sg.disconnected,
            );
        }
    }

    fn net_send_all_except(&mut self, msg: ServerMessage, except_client_handle: Index) {
        let net_msg = net::serialize(msg);
        for (client_handle, client) in self.sg.clients.iter_mut() {
            if client_handle == except_client_handle {
                continue;
            }
            Self::net_send(
                &net_msg,
                client_handle,
                client,
                self.gs,
                &mut self.sg.disconnected,
            );
        }
    }

    fn net_send_one(&mut self, msg: ServerMessage, client_handle: Index) {
        let net_msg = net::serialize(msg);
        let client = &mut self.sg.clients[client_handle];
        Self::net_send(
            &net_msg,
            client_handle,
            client,
            self.gs,
            &mut self.sg.disconnected,
        );
    }

    fn net_send(
        net_msg: &NetworkMessage,
        client_handle: Index,
        client: &mut RemoteClient,
        gs: &GameState,
        disconnected: &mut FnvHashSet<Index>,
    ) {
        let res = client.conn.send(net_msg);
        if let Err(e) = res {
            let index = client_handle.slot();
            let name = &gs.players[client.player_handle].name;
            dbg_logf!("Client #{index} {name:?} error when sending: {e:?}");
            disconnected.insert(client_handle);
        }
    }

    /// Accept human clients trying to connect.
    fn sys_net_accept(&mut self) {
        loop {
            match self.sg.listener.accept_conn() {
                Ok(conn) => {
                    // LATER Need a better handshake
                    // so the initial messages don't change between versions.
                    // Should contain only version and newer decides if compatible?
                    // Version format - major.minor.patch + string for forward compat?
                    // Look how comfy gets version from git.

                    // Create client and player.
                    let addr = conn.addr();
                    let client = RemoteClient::new(conn, Index::DANGLING);
                    let client_handle = self.sg.clients.insert(client);
                    let name = "unconnected".to_owned(); // TODO?
                    let player = Player::new(name, ClientType::Remote(client_handle));
                    let player_handle = self.gs.players.insert(player);
                    self.sg.clients[client_handle].player_handle = player_handle;

                    let index = client_handle.slot();
                    dbg_logf!("Connection accepted: {addr} -> client #{index}");

                    // Send init to new player (contains his index).
                    // Has to be the first message after connecting.
                    let init = self.build_init(player_handle);
                    let msg = ServerMessage::Init(init);
                    self.net_send_one(msg, client_handle);

                    // Send new player to everyone except the new player
                    let player_init = PlayerInit {
                        index: player_handle.slot(),
                        name: self.gs.players[player_handle].name.clone(),
                        // Currently we don't need to send score here
                        // because all fields are 0 but in the future
                        // some gamemodes might have a non-zero starting score
                        // (e.g. number of lives in survival modes).
                        score: self.gs.players[player_handle].score.clone(),
                    };
                    let msg = ServerMessage::AddPlayer(player_init);
                    self.net_send_all_except(msg, client_handle);

                    // Create vehicle, send to everyone
                    // LATER New players should spectate
                    self.spawn_vehicle(player_handle, true);

                    dbg_logf!("Client #{index} init sent");
                }
                Err(err) => match err.kind() {
                    ErrorKind::WouldBlock => {
                        break;
                    }
                    _ => panic!("network error (accept): {}", err),
                },
            }
        }
    }

    fn build_init(&self, player_handle: Index) -> Init {
        let players = self
            .gs
            .players
            .iter()
            .map(|(handle, player)| PlayerInit {
                index: handle.slot(),
                name: player.name.clone(),
                score: player.score.clone(),
            })
            .collect();

        let vehicles = self
            .gs
            .vehicles
            .iter()
            .map(|(handle, vehicle)| VehicleInit {
                index: handle.slot(),
                physics: EntityPhysics {
                    pos: vehicle.pos,
                    vel: vehicle.vel,
                    angle: vehicle.angle,
                    turn_rate: vehicle.turn_rate,
                },
                veh_type: vehicle.veh_type,
                turret_angle_current: vehicle.turret_angle_current,
                turret_angle_wanted: vehicle.turret_angle_wanted,
                spawn_time: vehicle.spawn_time,
                owner: vehicle.owner.slot(),
            })
            .collect();

        let projectiles = self
            .gs
            .projectiles
            .iter()
            .map(|(handle, projectile)| ProjectileInit {
                index: handle.slot(),
                physics: EntityPhysics {
                    pos: projectile.pos,
                    vel: projectile.vel,
                    angle: projectile.angle,
                    turn_rate: projectile.turn_rate,
                },
                weapon: projectile.weapon,
                explode_time: projectile.explode_time,
                owner: projectile.owner.slot(),
            })
            .collect();

        Init {
            sv_version: env!("GIT_VERSION").to_owned(),
            map_path: self.map.path.clone(),
            frame_num: self.gs.frame_num,
            game_time: self.gs.game_time,
            game_time_prev: self.gs.game_time_prev,
            dt: self.gs.dt,
            players,
            local_player1_index: player_handle.slot(),
            local_player2_index: None, // LATER(splitscreen)
            vehicles,
            projectiles,
        }
    }

    /// Add bot clients if necessary.
    fn sys_connect_bots(&mut self) {
        let humans = self.sg.clients.len();
        let bots_min = self.cvars.g_players_min.saturating_sub(humans);
        let bots_max = self.cvars.g_players_max.saturating_sub(humans);

        let spawns = self.map.spawns().len() as f32;
        let tiles = self.map.count_tiles() as f32;
        let bots_desired_by_spawns = spawns / self.cvars.bots_spawns_per_bot;
        let bots_desired_by_tiles = tiles / self.cvars.bots_tiles_per_bot;
        let bots_desired_float = (bots_desired_by_spawns + bots_desired_by_tiles) / 2.0;
        let mut bots_desired = bots_desired_float as usize;
        if bots_min < bots_max {
            bots_desired = bots_desired.clamped(bots_min, bots_max);
        } else {
            dbg_logf!("g_players_min >= g_players_max");
        }
        bots_desired = bots_desired.min(self.cvars.bots_max);

        if self.gs.frame_num == 1 {
            dbg_logf!(
                "{} spawns -> {} bots",
                self.map.spawns().len(),
                bots_desired_by_spawns,
            );
            dbg_logf!(
                "{} tiles -> {} bots",
                self.map.count_tiles(),
                bots_desired_by_tiles,
            );
            dbg_logf!("Desired bots: {bots_desired}");
            dbg_logf!("Spawns per bot: {}", spawns / bots_desired_float);
            dbg_logf!("Tiles per bot: {}", tiles / bots_desired_float);
        }

        let bots_current = self.gs.ais.len();
        if bots_current > bots_desired {
            let to_remove = bots_current - bots_desired;
            let mut ai_handles = self.gs.ais.collect_handles();
            for _ in 0..to_remove {
                let ai_handle = ai_handles.pop().unwrap();
                let player_handle = self.gs.ais[ai_handle].player;
                let name = self.gs.players[player_handle].name.clone();
                self.remove_player(player_handle);
                self.gs.ais.remove(ai_handle);
                let msg = ServerMessage::RemovePlayer {
                    index: player_handle.slot(),
                };
                self.net_send_all(msg);
                // LATER Unify which methods send to client and which don't.
                // E.g. now remove_player doesn't but spawn_vehicle does - error prone.

                let index = ai_handle.slot();
                dbg_logf!("Removed bot @{index} {name:?}");
            }
        } else if bots_current < bots_desired {
            for i in self.gs.ais.len()..bots_desired {
                let name = if i < BOT_NAMES.len() {
                    BOT_NAMES[i].to_owned()
                } else {
                    format!("Bot {}", i + 1)
                };

                let ai = Ai::new(Index::DANGLING);
                let ai_handle = self.gs.ais.insert(ai);
                let player = Player::new(name, ClientType::Ai(ai_handle));
                let player_handle = self.gs.players.insert(player);
                self.gs.ais[ai_handle].player = player_handle;

                // LATER Use spawns when bot AI actually works
                self.spawn_vehicle(player_handle, false);

                let index = ai_handle.slot();
                let name = &self.gs.players[player_handle].name;
                dbg_logf!("Added bot @{index} {name:?}");
            }
        }
    }

    /// Receive input and commands from remote clients.
    fn sys_net_receive(&mut self) {
        let mut reply_msgs = Vec::new();
        for (client_handle, client) in self.sg.clients.iter_mut() {
            let (msgs, closed) = client.conn.receive();

            for msg in msgs {
                match msg {
                    ClientMessage::Connect(connect) => {
                        let Connect {
                            cl_version,
                            name1,
                            name2,
                        } = connect;
                        let index = client_handle.slot();
                        dbg_logf!("Client #{index} connected: {} ", cl_version);
                        dbg_logf!("name1: {:?}", name1);
                        dbg_logf!("name2: {:?}", name2);

                        self.gs.players[client.player_handle].name = name1;
                    }
                    ClientMessage::Input(net_input) => {
                        let player = &mut self.gs.players[client.player_handle];
                        player.input = net_input;
                    }
                    ClientMessage::Chat(_) => unimplemented!(), // LATER
                    ClientMessage::Pause => {
                        self.sg.paused = !self.sg.paused;

                        let index = client_handle.slot();
                        dbg_logf!("Client #{index} paused -> {}", self.sg.paused);

                        let msg = ServerMessage::Paused(self.sg.paused);
                        reply_msgs.push(msg);
                    }
                    ClientMessage::Join => unimplemented!(), // LATER
                    ClientMessage::Observe => unimplemented!(), // LATER
                }
            }

            if closed {
                let index = client_handle.slot();
                dbg_logf!("Client #{index} disconnected when receiving");
                self.sg.disconnected.insert(client_handle);
            }
        }

        for msg in reply_msgs {
            self.net_send_all(msg);
        }
    }

    /// Send updates to all clients.
    fn sys_net_send_updates(&mut self) {
        let player_inputs = self
            .gs
            .players
            .iter()
            .map(|(handle, player)| InputUpdate {
                index: handle.slot(),
                net_input: player.input,
            })
            .collect();

        let vehicles = self
            .gs
            .vehicles
            .iter()
            .map(|(handle, vehicle)| VehicleUpdate {
                index: handle.slot(),
                physics: EntityPhysics {
                    pos: vehicle.pos,
                    vel: vehicle.vel,
                    angle: vehicle.angle,
                    turn_rate: vehicle.turn_rate,
                },
                turret_angle_current: vehicle.turret_angle_current,
                turret_angle_wanted: vehicle.turret_angle_wanted,
            })
            .collect();

        let projectiles = self
            .gs
            .projectiles
            .iter()
            .map(|(handle, projectile)| ProjectileUpdate {
                index: handle.slot(),
                physics: EntityPhysics {
                    pos: projectile.pos,
                    vel: projectile.vel,
                    angle: projectile.angle,
                    turn_rate: projectile.turn_rate,
                },
            })
            .collect();

        // Send debug items, then clear everything on the server (not just expired)
        // so it doesn't get sent again next frame.
        let debug_texts = DEBUG_TEXTS.with(|texts| {
            let mut texts = texts.borrow_mut();
            mem::take(&mut *texts)
        });
        let debug_texts_world = DEBUG_TEXTS_WORLD.with(|texts| {
            let mut texts = texts.borrow_mut();
            mem::take(&mut *texts)
        });
        let debug_shapes = DEBUG_SHAPES.with(|shapes| {
            let mut shapes = shapes.borrow_mut();
            mem::take(&mut *shapes)
        });

        let update_stats = self.sg.update_durations.get_stats().unwrap_or_default();
        let gamelogic_stats = self.sg.gamelogic_durations.get_stats().unwrap_or_default();
        let server_timings = ServerTimings {
            update_durations_avg: update_stats.0,
            update_durations_max: update_stats.1,
            gamelogic_durations_avg: gamelogic_stats.0,
            gamelogic_durations_max: gamelogic_stats.1,
            update_fps: self.sg.update_fps.get_fps(),
            gamelogic_fps: self.sg.gamelogic_fps.get_fps(),
        };

        let update = Update {
            frame_num: self.gs.frame_num,
            game_time: self.gs.game_time,
            game_time_prev: self.gs.game_time_prev,
            dt: self.gs.dt,
            player_inputs,
            vehicles,
            projectiles,
            debug_texts,
            debug_texts_world,
            debug_shapes,
            server_timings,
        };
        let msg = ServerMessage::Update(update);
        self.net_send_all(msg);
    }

    /// Remove data of disconnected clients, notify others.
    fn sys_net_disconnect(&mut self) {
        let handles = mem::take(&mut self.sg.disconnected); // Borrowck
        for client_handle in handles {
            let player_handle = self.sg.clients[client_handle].player_handle;
            let name = self.gs.players[player_handle].name.clone();
            self.remove_player(player_handle);

            self.sg.clients.remove(client_handle);

            let msg = ServerMessage::RemovePlayer {
                index: player_handle.slot(),
            };
            self.net_send_all(msg);

            let index = client_handle.slot();
            dbg_logf!("Client #{index} {name:?} disconnected");
        }
    }
}

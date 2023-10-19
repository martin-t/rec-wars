use crate::prelude::*;

impl FrameCtx<'_> {
    // LATER Init fns are here because we need them before a ClientCtx can be created.
    // They are not common code though, move them somewhere else.

    pub fn init_player(&mut self, init: PlayerInit) {
        let PlayerInit { index, name, score } = init;
        let mut player = Player::new(name, ClientType::Local);
        player.score = score;
        let (_player_handle, old) = self.gs.players.insert_at_slot(index, player);
        assert!(old.is_none());
    }

    pub fn init_vehicle(&mut self, init: VehicleInit) {
        let VehicleInit {
            index,
            physics:
                EntityPhysics {
                    pos,
                    angle,
                    vel,
                    turn_rate,
                },
            veh_type,
            turret_angle_current,
            turret_angle_wanted,
            spawn_time,
            owner,
        } = init;

        let owner = self.gs.players.slot_to_index(owner).unwrap();
        let mut vehicle = Vehicle::new(self.cvars, pos, angle, veh_type, spawn_time, owner);
        vehicle.vel = vel;
        vehicle.turn_rate = turn_rate;
        vehicle.turret_angle_current = turret_angle_current;
        vehicle.turret_angle_wanted = turret_angle_wanted;

        let (vehicle_handle, _old) = self.gs.vehicles.insert_at_slot(index, vehicle);

        self.gs.players[owner].vehicle = Some(vehicle_handle);
    }

    pub fn init_projectile(&mut self, init: ProjectileInit) {
        let ProjectileInit {
            index,
            weapon,
            physics:
                EntityPhysics {
                    pos,
                    vel,
                    angle,
                    turn_rate,
                },
            explode_time,
            owner,
        } = init;

        let owner = self.gs.players.slot_to_index(owner).unwrap();
        let projectile = Projectile {
            weapon,
            pos,
            vel,
            angle,
            turn_rate,
            explode_time,
            owner,
            target: None, // LATER Simulate homing missiles on client too?
        };
        let (_projectile_handle, old) = self.gs.projectiles.insert_at_slot(index, projectile);
        soft_assert!(old.is_none());
    }

    pub fn remove_player(&mut self, player_handle: Index) {
        self.gs
            .projectiles
            .retain(|_, proj| proj.owner != player_handle);
        // LATER This ignores gs.rail_hits because we're gonna change that anyway.
        self.gs.vehicles.retain(|_, veh| veh.owner != player_handle);
        self.gs.players.remove(player_handle);
    }

    pub fn update_score_kill(&mut self, attacker_handle: Index, victim_handle: Index) {
        let attacker = &mut self.gs.players[attacker_handle];
        if attacker_handle == victim_handle {
            attacker.score.suicides += 1;
        } else {
            attacker.score.kills += 1;
        }

        let victim = &mut self.gs.players[victim_handle];
        victim.score.deaths += 1; // All deaths, including suicides
    }

    pub fn sys_debug_examples(&self, offset: Vec2f) {
        if !self.cvars.d_examples {
            return;
        }

        dbg_world_textf!(offset, "ft: {}", self.gs.frame_num);
        dbg_world_textf!(offset + v!(0 15), "gt: {:.03}", self.gs.game_time);

        dbg_line!(
            v!(offset.x, offset.y + 25.0),
            v!(offset.x, offset.y + 25.0) + v!(50 0)
        );

        dbg_arrow!(v!(offset.x, offset.y + 50.0), v!(50, 0));
        dbg_arrow!(v!(offset.x, offset.y + 75.0), v!(25, -10));

        dbg_cross!(v!(offset.x, offset.y + 125.0));

        dbg_rot!(v!(offset.x, offset.y + 150.0), 0.0);
        dbg_rot!(v!(offset.x, offset.y + 175.0), 30.0_f64.to_radians());
    }
}

//! Rendering using the macroquad engine.

use std::{cmp::Reverse, str};

use macroquad::prelude::*;

use crate::{
    client::ClientMode,
    debug::{details::UniqueLines, DEBUG_SHAPES, DEBUG_TEXTS, DEBUG_TEXTS_WORLD},
    map::{SurfaceKind, TILE_SIZE},
    prelude::*,
};

// LATER clean up at least some of the casts here

impl Client {
    pub fn render(&mut self, cvars: &Cvars) {
        self.render_fps.tick(cvars.d_fps_period, self.real_time);
        let start = get_time();

        match &self.client_mode {
            ClientMode::Singleplayer { player_handle } => {
                self.render_viewport(cvars, *player_handle)
            }
            ClientMode::Splitscreen {
                render_targets,
                player_handles,
            } => {
                let rect = Rect::new(
                    0.0,
                    0.0,
                    self.viewport_size.x as f32,
                    self.viewport_size.y as f32,
                );
                let mut camera = Camera2D::from_display_rect(rect);

                // Macroquad bug https://github.com/not-fl3/macroquad/issues/171
                // This does not appear fixed in macroquad 0.4 despite the changelog claiming so:
                // https://macroquad.rs/articles/macroquad-0-4/#camera-consistency
                camera.zoom.y = -camera.zoom.y;

                camera.render_target = Some(render_targets.0.clone());
                set_camera(&camera);
                clear_background(BLANK);
                self.render_viewport(cvars, player_handles.0);

                camera.render_target = Some(render_targets.1.clone());
                set_camera(&camera);
                clear_background(BLANK);
                self.render_viewport(cvars, player_handles.1);

                set_default_camera();
                draw_texture(&render_targets.0.texture, 0.0, 0.0, WHITE);
                let offset_x = (self.viewport_size.x + cvars.r_splitscreen_gap) as f32;
                draw_texture(&render_targets.1.texture, offset_x, 0.0, WHITE);
            }
        }

        self.render_shared(cvars);

        let end = get_time();
        self.draw_calls_durations
            .add(cvars.d_timing_samples, end - start);
    }

    fn render_viewport(&self, cvars: &Cvars, local_player_handle: Index) {
        // This is one long function. A lot of people will tell you that's bad™
        // because they've heard it from other people who think long functions are bad™.
        // Most of those people haven't written a game bigger than snake. Carmack says it's ok so it's ok:
        // http://number-none.com/blow/blog/programming/2014/09/26/carmack-on-inlined-code.html

        let Client {
            assets,
            map,
            gs,
            cg,
            ..
        } = self;

        let player = &gs.players[local_player_handle];
        let player_vehicle = &gs.vehicles[player.vehicle.unwrap()];
        let player_entity_pos = if let Some(gm_handle) = player.guided_missile {
            gs.projectiles[gm_handle].pos
        } else {
            player_vehicle.pos
        };

        // Don't put the camera so close to the edge that it would render area outside the map.
        // Also properly handle maps smaller than view size. Note they can be smaller along X, Y or both.
        // Example maps for testing: Joust (2), extra/OK Corral (2)
        let map_size = map.maxs();
        let view_size = Vec2f::new(
            self.viewport_size.x.min(map_size.x),
            self.viewport_size.y.min(map_size.y),
        );
        let empty_space_size = self.viewport_size - view_size;
        let view_pos = empty_space_size / 2.0;

        // Camera center in world coords.
        let camera_center_min = view_size / 2.0;
        let camera_center_max = map_size - camera_center_min;
        let camera_center = player_entity_pos.clamped(camera_center_min, camera_center_max);

        // Position of the camera's top left corner in world coords.
        let camera_top_left = camera_center - camera_center_min;
        // Add this to world coords to get screen coords.
        // Forgetting to do this is a recurring source of bugs.
        // I've considered making a special type for screen coords (e.g. struct Vec2screen(Vec2f);)
        // so you couldn't accidentally pass world coords to drawing fns but it turned out to be more work than expected:
        // - The newtype had to manually impl all the needed operations of the underlying Vec2 type because ops don't autoderef.
        // - What would be the result of ops that take one world coord and one screen coord? Lots of cases to think about.
        // - Which type are sizes? Another type? E.g. `center = corner + size/2` makes sense in both screen and world coords.
        let camera_offset = -camera_top_left + view_pos;

        let top_left_tp = map.tile_pos(camera_top_left);
        let top_left_index = top_left_tp.index;
        let bg_offset = if cvars.r_align_to_pixels_background {
            top_left_tp.offset.floor()
        } else {
            top_left_tp.offset
        };

        // Draw non-walls
        let mut r = top_left_index.y;
        let mut y = -bg_offset.y;
        while y < view_size.y {
            let mut c = top_left_index.x;
            let mut x = -bg_offset.x;
            while x < view_size.x {
                let tile = map.col_row(c, r);

                if map.surface_of(tile).kind != SurfaceKind::Wall {
                    let img = &assets.texs_tiles[tile.surface_index];
                    render_tile(img, view_pos.x + x, view_pos.y + y, tile.angle);
                }

                c += 1;
                x += TILE_SIZE;
            }
            r += 1;
            y += TILE_SIZE;
        }

        // Helper to filter projectiles by weapon.
        let weapon_projectiles = |weapon| {
            gs.projectiles
                .iter()
                .filter(move |(_, proj)| proj.weapon == weapon)
        };

        let outside_view_top_left = view_pos - TILE_SIZE;
        let outside_view_bottom_right = view_pos + view_size + TILE_SIZE;
        // Is the object certainly outside camera view?
        // Only works on objects smaller that tile size, which is most.
        // Exceptions are lines and text.
        let cull = |scr_pos: Vec2f| {
            scr_pos.x < outside_view_top_left.x
                || scr_pos.y < outside_view_top_left.y
                || scr_pos.x > outside_view_bottom_right.x
                || scr_pos.y > outside_view_bottom_right.y
        };

        // Draw MGs
        for (_, mg) in weapon_projectiles(Weapon::Mg) {
            let scr_pos = mg.pos + camera_offset;
            if cull(scr_pos) {
                continue;
            }
            // we're drawing from the bullet's position backwards
            let scr_end = scr_pos - mg.vel.normalized() * cvars.cl_machine_gun_trail_length;
            render_line(
                scr_pos,
                scr_end,
                cvars.cl_machine_gun_trail_thickness,
                YELLOW,
            );
        }

        // Draw railguns
        for beam in &cg.rail_beams {
            let scr_begin = beam.begin + camera_offset;
            let scr_end = beam.end + camera_offset;
            render_line(
                scr_begin,
                scr_end,
                cvars.cl_railgun_trail_thickness,
                Color::new(0.0, 0.0, 1.0, 1.0),
            );
        }

        // Draw rockets, homing and guided missiles
        for (_, proj) in weapon_projectiles(Weapon::Rockets) {
            let scr_pos = proj.pos + camera_offset;
            if cull(scr_pos) {
                continue;
            }
            let offset = Vec2f::new(cvars.r_rockets_offset_x, cvars.r_rockets_offset_y);
            render_tex_offset(&assets.tex_rocket, scr_pos, proj.vel.to_angle(), offset);
        }
        for (_, proj) in weapon_projectiles(Weapon::Hm) {
            let scr_pos = proj.pos + camera_offset;
            if cull(scr_pos) {
                continue;
            }
            let offset = Vec2f::new(
                cvars.r_homing_missile_offset_x,
                cvars.r_homing_missile_offset_y,
            );
            render_tex_offset(&assets.tex_hm, scr_pos, proj.vel.to_angle(), offset);
        }
        for (_, proj) in weapon_projectiles(Weapon::Gm) {
            let scr_pos = proj.pos + camera_offset;
            if cull(scr_pos) {
                continue;
            }
            let offset = Vec2f::new(
                cvars.r_guided_missile_offset_x,
                cvars.r_guided_missile_offset_y,
            );
            render_tex_offset(&assets.tex_gm, scr_pos, proj.vel.to_angle(), offset);
        }

        // Draw BFGs
        // client.context.set_fill_style(&"lime".into());
        // client.context.set_stroke_style(&"lime".into());
        for (_, bfg) in weapon_projectiles(Weapon::Bfg) {
            let scr_pos = bfg.pos + camera_offset;
            if cull(scr_pos) {
                continue;
            }
            draw_circle(
                scr_pos.x as f32,
                scr_pos.y as f32,
                cvars.g_bfg_radius as f32,
                GREEN,
            );
            for (_, vehicle) in &self.gs.vehicles {
                // TODO This should be shared logic in BFG code
                if vehicle.owner == bfg.owner || vehicle.destroyed() {
                    continue;
                }
                // LATER Find nearest point on BFG's path?
                if weapons::bfg_beam_hit(cvars, map, bfg.pos, vehicle.pos) {
                    let scr_src = bfg.pos + camera_offset;
                    let scr_dest = vehicle.pos + camera_offset;
                    render_line(scr_src, scr_dest, 1.0, GREEN);
                }
            }
        }

        // Draw chassis
        for (_, vehicle) in &gs.vehicles {
            let scr_pos = vehicle.pos + camera_offset;
            if cull(scr_pos) {
                continue;
            }
            let img = if vehicle.destroyed() {
                &assets.texs_wrecks[vehicle.veh_type as usize]
            } else {
                &assets.texs_vehicles[vehicle.veh_type as usize * 2]
            };
            render_tex_center(img, scr_pos, vehicle.angle);
            // LATER draw hitboxes
            // if cvars.d_draw && cvars.d_draw_hitboxes {
            //     client.context.set_stroke_style(&"yellow".into());
            //     client.context.begin_path();
            //     let corners = vehicle.hitbox.corners(scr_pos, vehicle.angle);
            //     move_to(client, corners[0]);
            //     line_to(client, corners[1]);
            //     line_to(client, corners[2]);
            //     line_to(client, corners[3]);
            //     client.context.close_path();
            //     client.context.stroke();
            // }
        }

        // LATER Draw cow

        // Draw turrets
        for (_, vehicle) in &gs.vehicles {
            if vehicle.destroyed() {
                continue;
            }

            let vehicle_scr_pos = vehicle.pos + camera_offset;
            if cull(vehicle_scr_pos) {
                continue;
            }

            let img = &assets.texs_vehicles[vehicle.veh_type as usize * 2 + 1];
            let offset_chassis =
                vehicle.angle.to_mat2f() * cvars.g_vehicle_turret_offset_chassis(vehicle.veh_type);
            let turret_scr_pos = vehicle_scr_pos + offset_chassis;
            let offset_turret = cvars.g_vehicle_turret_offset_turret(vehicle.veh_type);
            render_tex_offset(
                img,
                turret_scr_pos,
                vehicle.angle + vehicle.turret_angle_current,
                offset_turret,
            );
        }

        // Draw explosions
        let iter: Box<dyn Iterator<Item = &Explosion>> = if cvars.r_explosions_reverse_order {
            Box::new(cg.explosions.iter().rev())
        } else {
            Box::new(cg.explosions.iter())
        };
        for explosion in iter {
            let scr_pos = explosion.pos + camera_offset;
            if cull(scr_pos) {
                continue;
            }

            // It looks like the original animation is made for 30 fps.
            // Single stepping a recording of the original RecWars explosion in blender:
            // 13 sprites, 31 frames - examples:
            //      2,2,3,1,3,3,2,3,2,2,3,2,3
            //      2,2,2,3,1,3,2,2,3,2,2,3,4
            // Different each time probably because RecWar's and the recorder's framerate don't match exactly.
            //
            // This code produces similar results,
            // though it might display a single sprite for 4 frames slightly more often.
            let progress = (gs.game_time - explosion.start_time) / cvars.r_explosion_duration;
            // 13 sprites in the sheet, 100x100 pixels per sprite
            let frame = (progress * 13.0).floor();
            let (offset, img);
            if explosion.bfg {
                offset = (12.0 - frame) * 100.0;
                img = &assets.tex_explosion_cyan;
            } else {
                offset = frame * 100.0;
                img = &assets.tex_explosion;
            };
            draw_texture_ex(
                img,
                (scr_pos.x - 50.0 * explosion.scale) as f32,
                (scr_pos.y - 50.0 * explosion.scale) as f32,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(
                        100.0 * explosion.scale as f32,
                        100.0 * explosion.scale as f32,
                    )),
                    source: Some(Rect::new(offset as f32, 0.0, 100.0, 100.0)),
                    ..Default::default()
                },
            );
        }

        // Draw walls
        // They are above explosions and turrets, just like in RecWar.
        let mut r = top_left_index.y;
        let mut y = -bg_offset.y;
        while y < view_size.y {
            let mut c = top_left_index.x;
            let mut x = -bg_offset.x;
            while x < view_size.x {
                let tile = map.col_row(c, r);

                if map.surface_of(tile).kind == SurfaceKind::Wall {
                    let img = &assets.texs_tiles[tile.surface_index];
                    render_tile(img, view_pos.x + x, view_pos.y + y, tile.angle);
                }

                c += 1;
                x += TILE_SIZE;
            }
            r += 1;
            y += TILE_SIZE;
        }

        // Draw cluster bombs
        // LATER what about shadows (in general)? Should they stack?
        if cvars.r_cluster_bombs {
            for (_, cb) in weapon_projectiles(Weapon::Cb) {
                let scr_pos = cb.pos + camera_offset;
                if cull(scr_pos) {
                    continue;
                }

                let corner = scr_pos - cvars.cl_cluster_bomb_size / 2.0;
                // Tecnically, we should draw all shadows first, then all the projectiles,
                // but actually it barely matters and I think RecWar does it this way too.
                draw_rectangle(
                    (corner.x + cvars.g_cluster_bomb_shadow_x) as f32,
                    (corner.y + cvars.g_cluster_bomb_shadow_y) as f32,
                    cvars.cl_cluster_bomb_size as f32,
                    cvars.cl_cluster_bomb_size as f32,
                    Color::new(0.0, 0.0, 0.0, cvars.g_cluster_bomb_shadow_alpha as f32),
                );
                draw_rectangle(
                    corner.x as f32,
                    corner.y as f32,
                    cvars.cl_cluster_bomb_size as f32,
                    cvars.cl_cluster_bomb_size as f32,
                    Color::new(0.0, 1.0, 1.0, 1.0),
                );
            }
        }

        // Draw world-space HUD elements:

        // Names
        if cvars.hud_names {
            for (_, vehicle) in &gs.vehicles {
                let scr_pos = vehicle.pos + camera_offset;
                if cull(scr_pos) {
                    // LATER, restrict name length
                    continue;
                }

                let name = &gs.players[vehicle.owner].name;
                let size = measure_text(name, None, cvars.hud_names_font_size as u16, 1.0);
                render_text_with_shadow(
                    cvars,
                    name,
                    scr_pos.x as f32 - size.width / 2.0,
                    (scr_pos.y + cvars.hud_names_y) as f32,
                    cvars.hud_names_font_size,
                    Color::new(
                        cvars.hud_names_brightness as f32,
                        cvars.hud_names_brightness as f32,
                        cvars.hud_names_brightness as f32,
                        cvars.hud_names_alpha as f32,
                    ),
                    cvars.hud_names_shadow_x,
                    cvars.hud_names_shadow_y,
                    cvars.hud_names_shadow_alpha,
                );
            }
        }

        // Homing missile indicators
        // LATER dashed lines (maybe use image or https://docs.rs/macroquad/0.4.1/src/macroquad/shapes.rs.html#180-204)
        for (_, vehicle) in &gs.vehicles {
            let scr_pos = vehicle.pos + camera_offset;
            if cull(scr_pos) {
                continue;
            }

            if !vehicle.hms.is_empty() {
                draw_circle_lines(
                    scr_pos.x as f32,
                    scr_pos.y as f32,
                    cvars.hud_missile_indicator_radius as f32,
                    1.0,
                    GREEN,
                );
            }
            for &hm_handle in &vehicle.hms {
                let hm = &gs.projectiles[hm_handle];
                let dir = (hm.pos - vehicle.pos).normalized();
                let end = scr_pos + dir * cvars.hud_missile_indicator_radius;
                render_line(scr_pos, end, 1.0, GREEN);
            }
        }

        // Spawn location indicator
        let alive_time = gs.game_time - player_vehicle.spawn_time;
        if alive_time < cvars.cl_spawn_indicator_duration {
            let vehicle_scr_pos = player_vehicle.pos + camera_offset;

            // Radius here is distance from the square's center to its side.
            let max_radius = cvars.cl_spawn_indicator_square_side_begin / 2.0;
            let min_radius = cvars.cl_spawn_indicator_square_side_end / 2.0;
            let fraction_complete =
                (alive_time / cvars.cl_spawn_indicator_animation_time).clamp(0.0, 1.0) as f32;
            let radius = (max_radius - min_radius) * (1.0 - fraction_complete) + min_radius;

            // Horizontal and vertical lines pointing at the vehicle.
            draw_line(
                0.0,
                vehicle_scr_pos.y as f32,
                vehicle_scr_pos.x as f32 - min_radius,
                vehicle_scr_pos.y as f32,
                cvars.cl_spawn_indicator_thickness,
                GREEN,
            );
            draw_line(
                vehicle_scr_pos.x as f32 + min_radius,
                vehicle_scr_pos.y as f32,
                self.viewport_size.x as f32,
                vehicle_scr_pos.y as f32,
                cvars.cl_spawn_indicator_thickness,
                GREEN,
            );
            draw_line(
                vehicle_scr_pos.x as f32,
                0.0,
                vehicle_scr_pos.x as f32,
                vehicle_scr_pos.y as f32 - min_radius,
                cvars.cl_spawn_indicator_thickness,
                GREEN,
            );
            draw_line(
                vehicle_scr_pos.x as f32,
                vehicle_scr_pos.y as f32 + min_radius,
                vehicle_scr_pos.x as f32,
                self.viewport_size.y as f32,
                cvars.cl_spawn_indicator_thickness,
                GREEN,
            );

            // Square with the vehicle in the center - first shrinks, then blinks.
            let period = cvars.cl_spawn_indicator_blinking_period;
            let still_shrinking = alive_time < cvars.cl_spawn_indicator_animation_time; // Don't blink during the animation
            let blinking_disabled = period == 0.0;
            let visible = alive_time % period < period / 2.0;
            if still_shrinking || blinking_disabled || visible {
                // We have to use thickness*2 here: https://github.com/not-fl3/macroquad/issues/271
                draw_rectangle_lines(
                    vehicle_scr_pos.x as f32 - radius,
                    vehicle_scr_pos.y as f32 - radius,
                    radius * 2.0,
                    radius * 2.0,
                    cvars.cl_spawn_indicator_thickness * 2.0,
                    GREEN,
                );
            }
        }

        // Deduplicate and draw debug shapes
        DEBUG_SHAPES.with_borrow_mut(|shapes| {
            // Sometimes debug shapes overlap and only the last one gets drawn.
            // This is especially common when both client and server wanna draw.
            // So instead, we convert everything to lines,
            // merge colors if they overlap and only then draw it.
            // This way if cl and sv shapes overlap, they end up yellow (red + green).
            let mut lines = UniqueLines::default();
            for shape in shapes.iter_mut() {
                if cvars.d_draw {
                    shape.to_lines(cvars, &mut lines);
                }
                // LATER This means debug shapes don't stay on during pause.
                //  Use spawn time like everything else.
                shape.time -= gs.dt;
            }
            for (_, line) in lines.0 {
                let scr_begin = line.begin + camera_offset;
                let scr_end = line.end + camera_offset;
                render_line(scr_begin, scr_end, cvars.d_draw_line_thickness, line.color);
            }
        });

        // Draw screen-space HUD elements:

        let mut player_points: Vec<_> = gs
            .players
            .iter()
            .map(|(index, player)| (index, player.score.points(cvars)))
            .collect();
        player_points.sort_by_key(|&(_, points)| Reverse(points));

        // Score
        let score_pos = hud_pos(view_pos, view_size, cvars.hud_score_x, cvars.hud_score_y);
        let points = player.score.points(cvars).to_string();
        render_text_with_shadow(
            cvars,
            &points,
            score_pos.x,
            score_pos.y,
            cvars.hud_score_font_size,
            WHITE,
            cvars.hud_score_shadow_x,
            cvars.hud_score_shadow_y,
            1.0,
        );

        // Ranking
        // Original RW shows "current rank / total players (+/- points difference to leader or second)"
        // as a big but not bold number with a 1px shadow. E.g. "1/3 (+5)" or "2/3 (0)".
        // There's no special treatement for players with the same number of points.
        let ranking_pos = hud_pos(
            view_pos,
            view_size,
            cvars.hud_ranking_x,
            cvars.hud_ranking_y,
        );
        let current_index = player_points
            .iter()
            .position(|&(handle, _)| handle == local_player_handle)
            .unwrap();
        let points_diff = if current_index == 0 {
            if player_points.len() == 1 {
                // The player is alone.
                0
            } else {
                player_points[current_index].1 - player_points[1].1
            }
        } else {
            player_points[current_index].1 - player_points[0].1
        };
        let ranking = if points_diff > 0 {
            // Only show the + sign for positive numbers, not 0
            format!(
                "{}/{} (+{})",
                current_index + 1,
                player_points.len(),
                points_diff
            )
        } else {
            format!(
                "{}/{} ({})",
                current_index + 1,
                player_points.len(),
                points_diff
            )
        };
        render_text_with_shadow(
            cvars,
            &ranking,
            ranking_pos.x,
            ranking_pos.y,
            cvars.hud_ranking_font_size,
            WHITE,
            cvars.hud_ranking_shadow_x,
            cvars.hud_ranking_shadow_y,
            1.0,
        );

        // Hit points (goes from green to red)
        // Might wanna use https://crates.io/crates/colorsys if I need more color operations.
        // Hit points to color (poor man's HSV):
        // 0.0 = red
        // 0.0..0.5 -> increase green channel
        // 0.5 = yellow
        // 0.5..1.0 -> decrease red channel
        // 1.0 = green
        let player_vehicle = &gs.vehicles[player.vehicle.unwrap()];
        let r = 1.0 - (player_vehicle.hp_fraction.clamped(0.5, 1.0) - 0.5) * 2.0;
        let g = player_vehicle.hp_fraction.clamped(0.0, 0.5) * 2.0;
        let rgb = Color::new(r as f32, g as f32, 0.0, 1.0);
        let hp_pos = hud_pos(view_pos, view_size, cvars.hud_hp_x, cvars.hud_hp_y);
        draw_rectangle(
            hp_pos.x,
            hp_pos.y,
            (cvars.hud_hp_width * player_vehicle.hp_fraction) as f32,
            cvars.hud_hp_height as f32,
            rgb,
        );
        if cvars.d_draw_texts && cvars.d_draw_hud {
            let hp_number =
                player_vehicle.hp_fraction * cvars.g_vehicle_hp(player_vehicle.veh_type);
            let hp_text = format!("{}", hp_number);
            render_text_with_shadow(
                cvars,
                &hp_text,
                hp_pos.x - 25.0,
                hp_pos.y + cvars.hud_hp_height as f32,
                16.0,
                RED,
                1.0,
                1.0,
                cvars.d_draw_text_shadow_alpha,
            );
        }

        // Ammo
        let ammo = player_vehicle.ammos[player.cur_weapon as usize];
        let ammo_fraction = match ammo {
            Ammo::Loaded(_ready_time, count) => {
                let max = cvars.g_weapon_reload_ammo(player.cur_weapon);
                count as f64 / max as f64
            }
            Ammo::Reloading(start, end) => {
                let max_diff = end - start;
                let cur_diff = gs.game_time - start;
                cur_diff / max_diff
            }
        };
        let ammo_pos = hud_pos(view_pos, view_size, cvars.hud_ammo_x, cvars.hud_ammo_y);
        draw_rectangle(
            ammo_pos.x,
            ammo_pos.y,
            (cvars.hud_ammo_width * ammo_fraction) as f32,
            cvars.hud_ammo_height as f32,
            YELLOW,
        );
        if cvars.d_draw_texts && cvars.d_draw_hud {
            let ammo_number = match ammo {
                Ammo::Loaded(_ready_time, count) => count,
                Ammo::Reloading(_start, _end) => 0,
            };
            render_text_with_shadow(
                cvars,
                &ammo_number.to_string(),
                ammo_pos.x - 25.0,
                ammo_pos.y + cvars.hud_ammo_height as f32,
                16.0,
                RED,
                1.0,
                1.0,
                cvars.d_draw_text_shadow_alpha,
            );
        }

        // Weapon icon
        // The original shadows were part of the image but this is good enough for now.
        let weap_img = &assets.texs_weapon_icons[player.cur_weapon as usize];
        let weap_icon_pos = hud_pos(
            view_pos,
            view_size,
            cvars.hud_weapon_icon_x,
            cvars.hud_weapon_icon_y,
        ) - Vec2::new(weap_img.width(), weap_img.height()) / 2.0;
        draw_texture(
            weap_img,
            weap_icon_pos.x + cvars.hud_weapon_icon_shadow_x,
            weap_icon_pos.y + cvars.hud_weapon_icon_shadow_y,
            Color::new(0.0, 0.0, 0.0, cvars.hud_weapon_icon_shadow_alpha as f32),
        );
        draw_texture(weap_img, weap_icon_pos.x, weap_icon_pos.y, WHITE);

        // Notifications
        let mut notification_y = if cvars.hud_notifications_y_from_center != 0.0 {
            screen_height() / 2.0 + cvars.hud_notifications_y_from_center
        } else {
            cvars.hud_notifications_y_from_top
        };
        let notification_most_recent = cg.notifications.last().map_or(0.0, |n| n.start_time);
        for notification in cg.notifications.iter().rev() {
            let age_current = gs.game_time - notification.start_time;
            let age_grow = cvars.hud_notifications_duration_grow;
            let age_large = age_grow + cvars.hud_notifications_duration_large;
            let age_shrink = age_large + cvars.hud_notifications_duration_shrink;

            let mut alpha = if age_current < age_grow {
                age_current / cvars.hud_notifications_duration_grow
            } else {
                let time_remaining = cvars.hud_notifications_duration - age_current;
                if time_remaining > cvars.hud_notifications_duration_fade_out {
                    1.0
                } else {
                    time_remaining / cvars.hud_notifications_duration_fade_out
                }
            } as f32;
            // The most recent "event" (sometimes multiple messages) should be fully visible,
            // the rest should be faded out a little.
            if notification.start_time != notification_most_recent {
                alpha *= cvars.hud_notifications_alpha_old;
            }
            let color = Color::new(
                notification.color.x,
                notification.color.y,
                notification.color.z,
                alpha,
            );

            let font_size = if age_current < age_grow {
                let factor = age_current / cvars.hud_notifications_duration_grow;
                Lerp::lerp(0.0, cvars.hud_notifications_font_size_large, factor)
            } else if age_current < age_large {
                cvars.hud_notifications_font_size_large
            } else if age_current < age_shrink {
                let factor = (age_current - age_large) / cvars.hud_notifications_duration_shrink;
                Lerp::lerp(
                    cvars.hud_notifications_font_size_large,
                    cvars.hud_notifications_font_size,
                    factor,
                )
            } else {
                cvars.hud_notifications_font_size
            };
            let measured_size = measure_text(&notification.text, None, font_size as u16, 1.0);
            render_text_with_shadow(
                cvars,
                &notification.text,
                (screen_width() - measured_size.width) / 2.0,
                notification_y + measured_size.height / 2.0,
                font_size,
                color,
                1.0,
                1.0,
                alpha,
            );
            notification_y += cvars.hud_notifications_y_offset;
        }

        // Scoreboard
        if player_vehicle.destroyed() {
            let width = cvars.hud_scoreboard_width_name
                + cvars.hud_scoreboard_width_kills
                + cvars.hud_scoreboard_width_deaths
                + cvars.hud_scoreboard_width_points;
            let height = (gs.players.len() + 1) as f32 * cvars.hud_scoreboard_line_height as f32;
            let x_start = view_pos.x as f32 + (view_size.x as f32 - width) / 2.0;
            let mut x = x_start.floor();
            let mut y = view_pos.y as f32 + (view_size.y as f32 - height) / 2.0;
            y = y.floor();

            let fs = cvars.hud_scoreboard_font_size;
            let sx = cvars.hud_scoreboard_shadow_x;
            let sy = cvars.hud_scoreboard_shadow_y;

            // LATER bold header
            render_text_with_shadow(cvars, "Name", x, y, fs, WHITE, sx, sy, 1.0);
            x += cvars.hud_scoreboard_width_name;
            render_text_with_shadow(cvars, "Kills", x, y, fs, WHITE, sx, sy, 1.0);
            x += cvars.hud_scoreboard_width_kills;
            render_text_with_shadow(cvars, "Deaths", x, y, fs, WHITE, sx, sy, 1.0);
            x += cvars.hud_scoreboard_width_deaths;
            render_text_with_shadow(cvars, "Points", x, y, fs, WHITE, sx, sy, 1.0);

            y += cvars.hud_scoreboard_line_height as f32;

            for (player_handle, points) in player_points {
                let color = if player_handle == local_player_handle {
                    WHITE
                } else {
                    Color::new(0.8, 0.8, 0.8, 1.0)
                };
                let player = &gs.players[player_handle];
                let name = &player.name;
                let kills = &player.score.kills.to_string();
                let deaths = &player.score.deaths.to_string();
                let points = &points.to_string();

                x = x_start;
                render_text_with_shadow(cvars, name, x, y, fs, color, sx, sy, 1.0);
                x += cvars.hud_scoreboard_width_name;
                render_text_with_shadow(cvars, kills, x, y, fs, color, sx, sy, 1.0);
                x += cvars.hud_scoreboard_width_kills;
                render_text_with_shadow(cvars, deaths, x, y, fs, color, sx, sy, 1.0);
                x += cvars.hud_scoreboard_width_deaths;
                render_text_with_shadow(cvars, points, x, y, fs, color, sx, sy, 1.0);

                y += cvars.hud_scoreboard_line_height as f32;
            }
        }

        // Clear background around the map if it's smaller than the screen.
        // This covers up any game entities which were drawn outside view.
        // It would also clear debug text from last frame if macroquad didn't do it automatically
        // (it calls clear_background at the begining of each frame anyway).
        if empty_space_size.x > 0.0 {
            // Draw 4 black stripes (rectangles) around view:
            // +----+------------+----+
            // |    |     3      |    |
            // |    +------------+    |
            // |    |            |    |
            // | 1  |    view    | 2  |
            // |    |            |    |
            // |    +------------+    |
            // |    |     4      |    |
            // +----+------------+----+
            // view_pos.x is width of vertical stripe
            // view_pos.y is height of horizontal stripe
            draw_rectangle(
                0.0,
                0.0,
                view_pos.x as f32,
                self.viewport_size.y as f32,
                BLACK,
            );
            draw_rectangle(
                (view_pos.x + view_size.x) as f32,
                0.0,
                view_pos.x as f32,
                self.viewport_size.y as f32,
                BLACK,
            );
            draw_rectangle(
                view_pos.x as f32,
                0.0,
                view_size.x as f32,
                view_pos.y as f32,
                BLACK,
            );
            draw_rectangle(
                view_pos.x as f32,
                (view_pos.y + view_size.y) as f32,
                view_size.x as f32,
                view_pos.y as f32,
                BLACK,
            );
        }

        // Pause
        if cg.paused {
            let paused_size = measure_text("PAUSED", None, cvars.hud_pause_font_size as u16, 1.0);
            render_text_with_shadow(
                cvars,
                "PAUSED",
                (view_size.x as f32 - paused_size.width) / 2.0 + view_pos.x as f32,
                (view_size.y as f32 - paused_size.height) / 2.0 + view_pos.y as f32,
                cvars.hud_pause_font_size,
                RED,
                cvars.hud_pause_shadow_x,
                cvars.hud_pause_shadow_y,
                1.0,
            );
        }

        // Draw world debug text
        DEBUG_TEXTS_WORLD.with_borrow(|texts| {
            if cvars.d_draw && cvars.d_draw_world_texts {
                for text in texts.iter() {
                    let scr_pos = text.pos + camera_offset;
                    if cull(scr_pos) {
                        // LATER Technically the text can be so long
                        // that it's culled overzealously but meh, perf is more important.
                        continue;
                    }

                    render_text_with_shadow(
                        cvars,
                        &text.msg,
                        scr_pos.x as f32,
                        scr_pos.y as f32,
                        16.0,
                        RED,
                        1.0,
                        1.0,
                        cvars.d_draw_text_shadow_alpha,
                    );
                }
            }
        });
    }

    fn render_shared(&self, cvars: &Cvars) {
        let screen_size = Vec2f::new(screen_width() as f64, screen_height() as f64);

        // Draw FPS
        if cvars.d_fps {
            let fps_pos = hud_pos(Vec2f::zero(), screen_size, cvars.d_fps_x, cvars.d_fps_y);
            render_text_with_shadow(
                cvars,
                &format!(
                    "client update FPS: {:.1}   gamelogic FPS: {:.1}   render FPS: {:.1}",
                    self.update_fps.get_fps(),
                    self.gamelogic_fps.get_fps(),
                    self.render_fps.get_fps()
                ),
                fps_pos.x - 120.0, // LATER remove the offset after finding a decent font
                fps_pos.y,
                16.0,
                RED,
                1.0,
                1.0,
                cvars.d_draw_text_shadow_alpha,
            );
        }

        // Draw server perf info
        if cvars.d_draw && cvars.d_draw_perf_server {
            let mut perf_pos = hud_pos(
                Vec2f::zero(),
                screen_size,
                cvars.hud_perf_server_x,
                cvars.hud_perf_server_y,
            );
            render_text_with_shadow(
                cvars,
                &format!("last {} server frames (in ms):", cvars.d_timing_samples),
                perf_pos.x,
                perf_pos.y,
                16.0,
                RED,
                1.0,
                1.0,
                cvars.d_draw_text_shadow_alpha,
            );
            perf_pos.y += 15.0;
            let text = format!(
                "update avg: {:.1}, max: {:.1}",
                self.cg.server_timings.update_durations_avg * 1000.0,
                self.cg.server_timings.update_durations_max * 1000.0
            );
            render_text_with_shadow(
                cvars,
                &text,
                perf_pos.x,
                perf_pos.y,
                16.0,
                RED,
                1.0,
                1.0,
                cvars.d_draw_text_shadow_alpha,
            );
            perf_pos.y += 15.0;
            let text = format!(
                "  gamelogic avg: {:.1}, max: {:.1}",
                self.cg.server_timings.gamelogic_durations_avg * 1000.0,
                self.cg.server_timings.gamelogic_durations_max * 1000.0
            );
            render_text_with_shadow(
                cvars,
                &text,
                perf_pos.x,
                perf_pos.y,
                16.0,
                RED,
                1.0,
                1.0,
                cvars.d_draw_text_shadow_alpha,
            );
            perf_pos.y += 15.0;
            let text = format!("update FPS: {:.1}", self.cg.server_timings.update_fps);
            render_text_with_shadow(
                cvars,
                &text,
                perf_pos.x,
                perf_pos.y,
                16.0,
                RED,
                1.0,
                1.0,
                cvars.d_draw_text_shadow_alpha,
            );
            perf_pos.y += 15.0;
            let text = format!("gamelogic FPS: {:.1}", self.cg.server_timings.gamelogic_fps);
            render_text_with_shadow(
                cvars,
                &text,
                perf_pos.x,
                perf_pos.y,
                16.0,
                RED,
                1.0,
                1.0,
                cvars.d_draw_text_shadow_alpha,
            );
        }

        // Draw client perf info
        if cvars.d_draw && cvars.d_draw_perf_client {
            let mut perf_pos = hud_pos(
                Vec2f::zero(),
                screen_size,
                cvars.hud_perf_client_x,
                cvars.hud_perf_client_y,
            );
            render_text_with_shadow(
                cvars,
                &format!("last {} client frames (in ms):", cvars.d_timing_samples),
                perf_pos.x,
                perf_pos.y,
                16.0,
                RED,
                1.0,
                1.0,
                cvars.d_draw_text_shadow_alpha,
            );
            perf_pos.y += 15.0;
            if let Some((avg, max)) = self.update_durations.get_stats() {
                let text = format!("update avg: {:.1}, max: {:.1}", avg * 1000.0, max * 1000.0);
                render_text_with_shadow(
                    cvars,
                    &text,
                    perf_pos.x,
                    perf_pos.y,
                    16.0,
                    RED,
                    1.0,
                    1.0,
                    cvars.d_draw_text_shadow_alpha,
                );
                perf_pos.y += 15.0;
            }
            if let Some((avg, max)) = self.gamelogic_durations.get_stats() {
                let text = format!(
                    "  gamelogic avg: {:.1}, max: {:.1}",
                    avg * 1000.0,
                    max * 1000.0
                );
                render_text_with_shadow(
                    cvars,
                    &text,
                    perf_pos.x,
                    perf_pos.y,
                    16.0,
                    RED,
                    1.0,
                    1.0,
                    cvars.d_draw_text_shadow_alpha,
                );
                perf_pos.y += 15.0;
            }
            if let Some((avg, max)) = self.draw_calls_durations.get_stats() {
                let text = format!(
                    "render cmds avg: {:.1}, max: {:.1}",
                    avg * 1000.0,
                    max * 1000.0
                );
                render_text_with_shadow(
                    cvars,
                    &text,
                    perf_pos.x,
                    perf_pos.y,
                    16.0,
                    RED,
                    1.0,
                    1.0,
                    cvars.d_draw_text_shadow_alpha,
                );
                perf_pos.y += 15.0;
            }
            if let Some((avg, max)) = self.screenshot_durations.get_stats() {
                let text = format!(
                    "screenshot avg: {:.1}, max: {:.1}",
                    avg * 1000.0,
                    max * 1000.0
                );
                render_text_with_shadow(
                    cvars,
                    &text,
                    perf_pos.x,
                    perf_pos.y,
                    16.0,
                    RED,
                    1.0,
                    1.0,
                    cvars.d_draw_text_shadow_alpha,
                );
                perf_pos.y += 15.0;
            }
            if let Some((avg, max)) = self.engine_durations.get_stats() {
                let text = format!(
                    "engine+rest avg: {:.1}, max: {:.1}",
                    avg * 1000.0,
                    max * 1000.0
                );
                render_text_with_shadow(
                    cvars,
                    &text,
                    perf_pos.x,
                    perf_pos.y,
                    16.0,
                    RED,
                    1.0,
                    1.0,
                    cvars.d_draw_text_shadow_alpha,
                );
            }
        }

        // Draw last key
        if cvars.d_last_key {
            if let Some(key_code) = self.last_key {
                render_text_with_shadow(
                    cvars,
                    &format!("{:?}", key_code),
                    300.0,
                    300.0,
                    32.0,
                    RED,
                    1.0,
                    1.0,
                    cvars.d_draw_text_shadow_alpha,
                );
            }
        }

        // Draw debug text
        let mut y = 25.0;
        DEBUG_TEXTS.with_borrow(|texts| {
            if cvars.d_draw && cvars.d_draw_texts {
                for text in texts.iter() {
                    render_text_with_shadow(
                        cvars,
                        text,
                        20.0,
                        y as f32,
                        16.0,
                        RED,
                        1.0,
                        1.0,
                        cvars.d_draw_text_shadow_alpha,
                    );
                    y += cvars.d_draw_texts_line_height;
                }
            }
        });
    }
}

/// Place the image's *center* at `scr_pos`,
/// rotate it clockwise by `angle`.
///
/// See Vec2f for more about the coord system and rotations.
fn render_tex_center(img: &Texture2D, pos: Vec2f, angle: f64) {
    draw_texture_ex(
        img,
        pos.x as f32 - img.width() / 2.0,
        pos.y as f32 - img.height() / 2.0,
        WHITE,
        DrawTextureParams {
            rotation: angle as f32,
            ..Default::default()
        },
    );
}

/// Place the `img`'s *center of rotation* at `scr_pos`,
/// rotate it clockwise by `angle`.
/// The center of rotation is `img`'s center + `offset`.
///
/// See Vec2f for more about the coord system and rotations.
fn render_tex_offset(img: &Texture2D, pos: Vec2f, angle: f64, offset: Vec2f) {
    draw_texture_ex(
        img,
        // This is effectively `pos - (offset + half_size)`, just written differently.
        (pos.x - offset.x) as f32 - img.width() / 2.0,
        (pos.y - offset.y) as f32 - img.height() / 2.0,
        WHITE,
        DrawTextureParams {
            rotation: angle as f32,
            pivot: Some(Vec2::new(pos.x as f32, pos.y as f32)),
            ..Default::default()
        },
    );
}

fn render_tile(img: &Texture2D, x: f64, y: f64, angle: f64) {
    draw_texture_ex(
        img,
        x as f32,
        y as f32,
        WHITE,
        DrawTextureParams {
            rotation: angle as f32,
            ..Default::default()
        },
    );
}

fn render_line(src: Vec2f, dest: Vec2f, thickness: f64, color: Color) {
    macroquad::shapes::draw_line(
        src.x as f32,
        src.y as f32,
        dest.x as f32,
        dest.y as f32,
        thickness as f32,
        color,
    );
}

#[allow(clippy::too_many_arguments)]
fn render_text_with_shadow(
    cvars: &Cvars,
    text: &str,
    mut x: f32,
    mut y: f32,
    font_size: f64,
    color: Color,
    shadow_offset_x: f32,
    shadow_offset_y: f32,
    shadow_alpha: f32,
) {
    if cvars.r_align_to_pixels_text {
        x = x.floor();
        y = y.floor();
    }
    if shadow_offset_x != 0.0 || shadow_offset_y != 0.0 {
        draw_text(
            text,
            x + shadow_offset_x,
            y + shadow_offset_y,
            font_size as f32,
            Color::new(0.0, 0.0, 0.0, shadow_alpha),
        );
    }
    draw_text(text, x, y, font_size as f32, color);
}

/// If x or y are negative, count them from the right or bottom respectively.
/// Useful to make HUD config cvars work for any screen/view size.
fn hud_pos(rect_pos: Vec2f, rect_size: Vec2f, mut x: f64, mut y: f64) -> Vec2 {
    if x < 0.0 {
        x += rect_size.x;
    }
    if y < 0.0 {
        y += rect_size.y;
    }
    Vec2::new((rect_pos.x + x) as f32, (rect_pos.y + y) as f32)
}

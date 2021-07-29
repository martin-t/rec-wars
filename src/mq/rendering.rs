//! Rendering using the macroquad engine.

use std::{cmp::Reverse, str};

use macroquad::prelude::*;
use thunderdome::Index;
use vek::Clamp;

use rec_wars::{
    cvars::Cvars,
    debugging::{DEBUG_CROSSES, DEBUG_LINES, DEBUG_TEXTS, DEBUG_TEXTS_WORLD},
    entities::{Ammo, Weapon},
    game_state::Explosion,
    map::{F64Ext, Kind, Vec2f, VecExt, TILE_SIZE},
    server::Server,
};

use crate::mq::{ClientMode, MacroquadClient};

// LATER when raw_canvas is removed, clean up all the casts here

pub(crate) fn render(client: &MacroquadClient, server: &Server, cvars: &Cvars) {
    match client.client_mode {
        ClientMode::Singleplayer { player_handle } => {
            render_viewport(client, server, cvars, player_handle)
        }
        ClientMode::Splitscreen {
            render_targets,
            player_handles,
        } => {
            let rect = Rect::new(
                0.0,
                0.0,
                client.viewport_size.x as f32,
                client.viewport_size.y as f32,
            );
            let mut camera = Camera2D::from_display_rect(rect);
            camera.zoom.y = -camera.zoom.y; // Macroquad bug https://github.com/not-fl3/macroquad/issues/171

            camera.render_target = Some(render_targets.0);
            set_camera(&camera);
            clear_background(BLANK);
            render_viewport(client, server, cvars, player_handles.0);

            camera.render_target = Some(render_targets.1);
            set_camera(&camera);
            clear_background(BLANK);
            render_viewport(client, server, cvars, player_handles.1);

            set_default_camera();
            draw_texture(render_targets.0.texture, 0.0, 0.0, WHITE);
            let offset_x = (client.viewport_size.x + cvars.r_splitscreen_gap) as f32;
            draw_texture(render_targets.1.texture, offset_x, 0.0, WHITE);
        }
    }

    render_shared(client, server, cvars);
}

fn render_viewport(
    client: &MacroquadClient,
    server: &Server,
    cvars: &Cvars,
    local_player_handle: Index,
) {
    let player = &server.gs.players[local_player_handle];
    let player_vehicle = &server.gs.vehicles[player.vehicle.unwrap()];
    let player_entity_pos = if let Some(gm_handle) = player.guided_missile {
        server.gs.projectiles[gm_handle].pos
    } else {
        player_vehicle.pos
    };

    // Don't put the camera so close to the edge that it would render area outside the map.
    // Also properly handle maps smaller than view size. Note they can be smaller along X, Y or both.
    // Example maps for testing: Joust (2), extra/OK Corral (2)
    let map_size = server.map.maxs();
    let view_size = Vec2f::new(
        client.viewport_size.x.min(map_size.x),
        client.viewport_size.y.min(map_size.y),
    );
    let empty_space_size = client.viewport_size - view_size;
    let view_pos = empty_space_size / 2.0;

    // Camera center in world coords.
    let camera_pos_min = view_size / 2.0;
    let camera_pos_max = map_size - camera_pos_min;
    let camera_pos = player_entity_pos.clamped(camera_pos_min, camera_pos_max);

    // Position of the camera's top left corner in world coords.
    let camera_top_left = camera_pos - camera_pos_min;
    // Add this to world coords to get screen coords.
    // Forgetting to do this is a recurring source of bugs.
    // I've considered making a special type for screen coords (e.g. struct Vec2screen(Vec2f);)
    // so you couldn't accidentally pass world coords to drawing fns but it turned out to be more work than expected:
    // - The newtype had to manually impl all the needed operations of the underlying Vec2 type because ops don't autoderef.
    // - What would be the result of ops that take one world coord and one screen coord? Lots of cases to think about.
    // - Which type are sizes? Another type? E.g. `center = corner + size/2` makes sense in both screen and world coords.
    let camera_offset = -camera_top_left + view_pos;

    let top_left_tp = server.map.tile_pos(camera_top_left);
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
            let tile = server.map.col_row(c, r);

            if server.map.surface_of(tile).kind != Kind::Wall {
                let img = client.imgs_tiles[tile.surface_index];
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
        server
            .gs
            .projectiles
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
        let scr_end = scr_pos - mg.vel.normalized() * cvars.g_machine_gun_trail_length;
        render_line(scr_pos, scr_end, 1.0, YELLOW);
    }

    // Draw railguns
    for beam in &server.gs.rail_beams {
        let scr_begin = beam.begin + camera_offset;
        let scr_end = beam.end + camera_offset;
        render_line(scr_begin, scr_end, 1.0, Color::new(0.0, 0.0, 1.0, 1.0));
    }

    // Draw rockets, homing and guided missiles
    for (_, proj) in weapon_projectiles(Weapon::Rockets) {
        let scr_pos = proj.pos + camera_offset;
        if cull(scr_pos) {
            continue;
        }
        render_img_center(client.img_rocket, scr_pos, proj.vel.to_angle());
    }
    for (_, proj) in weapon_projectiles(Weapon::Hm) {
        let scr_pos = proj.pos + camera_offset;
        if cull(scr_pos) {
            continue;
        }
        render_img_center(client.img_hm, scr_pos, proj.vel.to_angle());
    }
    for (_, proj) in weapon_projectiles(Weapon::Gm) {
        let scr_pos = proj.pos + camera_offset;
        if cull(scr_pos) {
            continue;
        }
        render_img_center(client.img_gm, scr_pos, proj.vel.to_angle());
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
    }
    for &(src, dest) in &server.gs.bfg_beams {
        let scr_src = src + camera_offset;
        let scr_dest = dest + camera_offset;
        render_line(scr_src, scr_dest, 1.0, GREEN);
    }

    // Draw chassis
    for (_, vehicle) in server.gs.vehicles.iter() {
        let scr_pos = vehicle.pos + camera_offset;
        if cull(scr_pos) {
            continue;
        }
        let img;
        if vehicle.destroyed() {
            img = client.imgs_wrecks[vehicle.veh_type as usize];
        } else {
            img = client.imgs_vehicles[vehicle.veh_type as usize * 2];
        }
        render_img_center(img, scr_pos, vehicle.angle);
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

    // TODO Draw cow

    // Draw turrets
    for (_, vehicle) in server.gs.vehicles.iter() {
        if vehicle.destroyed() {
            continue;
        }

        let scr_pos = vehicle.pos + camera_offset;
        if cull(scr_pos) {
            continue;
        }

        let img = client.imgs_vehicles[vehicle.veh_type as usize * 2 + 1];
        let offset_chassis =
            vehicle.angle.to_mat2f() * cvars.g_vehicle_turret_offset_chassis(vehicle.veh_type);
        let turret_scr_pos = scr_pos + offset_chassis;
        let offset_turret = cvars.g_vehicle_turret_offset_turret(vehicle.veh_type);
        render_img_offset(
            img,
            turret_scr_pos,
            vehicle.angle + vehicle.turret_angle_current,
            offset_turret,
        );
    }

    // Draw explosions
    let iter: Box<dyn Iterator<Item = &Explosion>> = if cvars.r_explosions_reverse_order {
        Box::new(server.gs.explosions.iter().rev())
    } else {
        Box::new(server.gs.explosions.iter())
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
        let progress = (server.gs.game_time - explosion.start_time) / cvars.r_explosion_duration;
        // 13 sprites in the sheet, 100x100 pixels per sprite
        let frame = (progress * 13.0).floor();
        let (offset, img);
        if explosion.bfg {
            offset = (12.0 - frame) * 100.0;
            img = client.img_explosion_cyan;
        } else {
            offset = frame * 100.0;
            img = client.img_explosion;
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
            let tile = server.map.col_row(c, r);

            if server.map.surface_of(tile).kind == Kind::Wall {
                let img = client.imgs_tiles[tile.surface_index];
                render_tile(img, view_pos.x + x, view_pos.y + y, tile.angle);
            }

            c += 1;
            x += TILE_SIZE;
        }
        r += 1;
        y += TILE_SIZE;
    }

    // Draw cluster bombs
    // TODO what about shadows (in general)?
    if cvars.r_draw_cluster_bombs {
        for (_, cb) in weapon_projectiles(Weapon::Cb) {
            let scr_pos = cb.pos + camera_offset;
            if cull(scr_pos) {
                continue;
            }

            let corner = scr_pos - cvars.g_cluster_bomb_size / 2.0;
            // Tecnically, we should draw all shadows first, then all the projectiles,
            // but actually it barely matters and I think RecWar does it this way too.
            draw_rectangle(
                (corner.x + cvars.g_cluster_bomb_shadow_x) as f32,
                (corner.y + cvars.g_cluster_bomb_shadow_y) as f32,
                cvars.g_cluster_bomb_size as f32,
                cvars.g_cluster_bomb_size as f32,
                Color::new(0.0, 0.0, 0.0, cvars.g_cluster_bomb_shadow_alpha as f32),
            );
            draw_rectangle(
                corner.x as f32,
                corner.y as f32,
                cvars.g_cluster_bomb_size as f32,
                cvars.g_cluster_bomb_size as f32,
                Color::new(0.0, 1.0, 1.0, 1.0),
            );
        }
    }

    // Draw world-space HUD elements:

    // Names
    if cvars.hud_names {
        for (_, vehicle) in server.gs.vehicles.iter() {
            let scr_pos = vehicle.pos + camera_offset;
            if cull(scr_pos) {
                // LATER, restrict name length
                continue;
            }

            let name = &server.gs.players[vehicle.owner].name;
            let size = measure_text(name, None, cvars.hud_names_font_size as u16, 1.0);
            // LATER remove cvars.hud_names_shadow_x/y when raw_canvas is removed
            render_text_with_shadow(
                &cvars,
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
                cvars.hud_names_shadow_mq_x,
                cvars.hud_names_shadow_mq_y,
                cvars.hud_names_shadow_alpha,
            );
        }
    }

    // Homing missile indicator
    // TODO dashed lines (maybe use image)
    let player_veh_scr_pos = player_vehicle.pos + camera_offset;
    draw_circle_lines(
        player_veh_scr_pos.x as f32,
        player_veh_scr_pos.y as f32,
        cvars.hud_missile_indicator_radius as f32,
        1.0,
        GREEN,
    );
    let dir = 0.0.to_vec2f(); // TODO
    let end = player_veh_scr_pos + dir * cvars.hud_missile_indicator_radius;
    render_line(player_veh_scr_pos, end, 1.0, GREEN);

    // Spawn location indicator
    if server.gs.game_time - player_vehicle.spawn_time < cvars.cl_spawn_indicator_duration {
        let vehicle_scr_pos = player_vehicle.pos + camera_offset;
        draw_line(
            0.0,
            vehicle_scr_pos.y as f32,
            client.viewport_size.x as f32,
            vehicle_scr_pos.y as f32,
            cvars.cl_spawn_indicator_thickness,
            GREEN,
        );
        draw_line(
            vehicle_scr_pos.x as f32,
            0.0,
            vehicle_scr_pos.x as f32,
            client.viewport_size.y as f32,
            cvars.cl_spawn_indicator_thickness,
            GREEN,
        );
    }

    // Debug lines and crosses
    // LATER colors (also in other places below)
    //  It would be nice to use MQ's Color struct in debugging but then everything depends on MQ.
    DEBUG_LINES.with(|lines| {
        let mut lines = lines.borrow_mut();
        for line in lines.iter_mut() {
            if cvars.d_draw && cvars.d_draw_lines {
                let scr_begin = line.begin + camera_offset;
                let scr_end = line.end + camera_offset;
                render_line(scr_begin, scr_end, 1.0, RED);
                if cvars.d_draw_lines_ends_length > 0.0 {
                    let segment = line.end - line.begin;
                    let perpendicular = Vec2f::new(-segment.y, segment.x).normalized();
                    render_line(
                        scr_begin - perpendicular * cvars.d_draw_lines_ends_length,
                        scr_begin + perpendicular * cvars.d_draw_lines_ends_length,
                        1.0,
                        RED,
                    );
                    render_line(
                        scr_end - perpendicular * cvars.d_draw_lines_ends_length,
                        scr_end + perpendicular * cvars.d_draw_lines_ends_length,
                        1.0,
                        RED,
                    );
                }
            }
            line.time -= server.gs.dt;
        }
    });
    DEBUG_CROSSES.with(|crosses| {
        let mut crosses = crosses.borrow_mut();
        for cross in crosses.iter_mut() {
            if cvars.d_draw && cvars.d_draw_crosses {
                let scr_point = cross.point + camera_offset;
                if cull(scr_point) {
                    continue;
                }

                let top_left = scr_point - Vec2f::new(-3.0, -3.0);
                let bottom_right = scr_point - Vec2f::new(3.0, 3.0);
                let top_right = scr_point - Vec2f::new(3.0, -3.0);
                let bottom_left = scr_point - Vec2f::new(-3.0, 3.0);
                render_line(top_left, bottom_right, 1.0, RED);
                render_line(top_right, bottom_left, 1.0, RED);
            }
            cross.time -= server.gs.dt;
        }
    });

    // Draw screen-space HUD elements:

    let mut player_points: Vec<_> = server
        .gs
        .players
        .iter()
        .map(|(index, player)| (index, player.score.points(&cvars)))
        .collect();
    player_points.sort_by_key(|&(_, points)| Reverse(points));

    // Score
    let score_pos = hud_pos(view_pos, view_size, cvars.hud_score_x, cvars.hud_score_y);
    let points = player.score.points(&cvars).to_string();
    render_text_with_shadow(
        &cvars,
        &points,
        score_pos.x,
        score_pos.y,
        cvars.hud_score_font_size,
        WHITE,
        cvars.hud_score_shadow_mq_x,
        cvars.hud_score_shadow_mq_y,
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
        &cvars,
        &ranking,
        ranking_pos.x,
        ranking_pos.y,
        cvars.hud_ranking_font_size,
        WHITE,
        cvars.hud_ranking_shadow_mq_x,
        cvars.hud_ranking_shadow_mq_y,
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
    let player_vehicle = &server.gs.vehicles[player.vehicle.unwrap()];
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
    if cvars.d_draw_text {
        let hp_number = player_vehicle.hp_fraction * cvars.g_vehicle_hp(player_vehicle.veh_type);
        let hp_text = format!("{}", hp_number);
        render_text_with_shadow(
            &cvars,
            &hp_text,
            hp_pos.x - 25.0,
            hp_pos.y + cvars.hud_hp_height as f32,
            16.0,
            RED,
            1.0,
            1.0,
            0.5,
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
            let cur_diff = server.gs.game_time - start;
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
    if cvars.d_draw_text {
        let ammo_number = match ammo {
            Ammo::Loaded(_ready_time, count) => count,
            Ammo::Reloading(_start, _end) => 0,
        };
        render_text_with_shadow(
            &cvars,
            &ammo_number.to_string(),
            ammo_pos.x - 25.0,
            ammo_pos.y + cvars.hud_ammo_height as f32,
            16.0,
            RED,
            1.0,
            1.0,
            0.5,
        );
    }

    // Weapon icon
    // The original shadows were part of the image but this is good enough for now.
    let weap_img = client.imgs_weapon_icons[player.cur_weapon as usize];
    let weap_icon_pos = hud_pos(
        view_pos,
        view_size,
        cvars.hud_weapon_icon_x,
        cvars.hud_weapon_icon_y,
    ) - Vec2::new(weap_img.width(), weap_img.height()) / 2.0;
    draw_texture(
        weap_img,
        weap_icon_pos.x + cvars.hud_weapon_icon_shadow_mq_x,
        weap_icon_pos.y + cvars.hud_weapon_icon_shadow_mq_y,
        Color::new(0.0, 0.0, 0.0, cvars.hud_weapon_icon_shadow_alpha as f32),
    );
    draw_texture(weap_img, weap_icon_pos.x, weap_icon_pos.y, WHITE);

    // Scoreboard
    if player_vehicle.destroyed() {
        let width = cvars.hud_scoreboard_width_name
            + cvars.hud_scoreboard_width_kills
            + cvars.hud_scoreboard_width_deaths
            + cvars.hud_scoreboard_width_points;
        let height = (server.gs.players.len() + 1) as f32 * cvars.hud_scoreboard_line_height as f32;
        let x_start = view_pos.x as f32 + (view_size.x as f32 - width) / 2.0;
        let mut x = x_start.floor();
        let mut y = view_pos.y as f32 + (view_size.y as f32 - height) / 2.0;
        y = y.floor();

        let fs = cvars.hud_scoreboard_font_size;
        let sx = cvars.hud_scoreboard_shadow_mq_x;
        let sy = cvars.hud_scoreboard_shadow_mq_y;

        // LATER bold header
        render_text_with_shadow(&cvars, "Name", x, y, fs, WHITE, sx, sy, 1.0);
        x += cvars.hud_scoreboard_width_name;
        render_text_with_shadow(&cvars, "Kills", x, y, fs, WHITE, sx, sy, 1.0);
        x += cvars.hud_scoreboard_width_kills;
        render_text_with_shadow(&cvars, "Deaths", x, y, fs, WHITE, sx, sy, 1.0);
        x += cvars.hud_scoreboard_width_deaths;
        render_text_with_shadow(&cvars, "Points", x, y, fs, WHITE, sx, sy, 1.0);

        y += cvars.hud_scoreboard_line_height as f32;

        for (player_handle, points) in player_points {
            let color = if player_handle == local_player_handle {
                WHITE
            } else {
                Color::new(0.8, 0.8, 0.8, 1.0)
            };
            let player = &server.gs.players[player_handle];
            let name = &player.name;
            let kills = &player.score.kills.to_string();
            let deaths = &player.score.deaths.to_string();
            let points = &points.to_string();

            x = x_start;
            render_text_with_shadow(&cvars, name, x, y, fs, color, sx, sy, 1.0);
            x += cvars.hud_scoreboard_width_name;
            render_text_with_shadow(&cvars, kills, x, y, fs, color, sx, sy, 1.0);
            x += cvars.hud_scoreboard_width_kills;
            render_text_with_shadow(&cvars, deaths, x, y, fs, color, sx, sy, 1.0);
            x += cvars.hud_scoreboard_width_deaths;
            render_text_with_shadow(&cvars, points, x, y, fs, color, sx, sy, 1.0);

            y += cvars.hud_scoreboard_line_height as f32;
        }
    }

    // Pause
    if server.paused {
        let paused_size = measure_text("PAUSED", None, cvars.hud_pause_font_size as u16, 1.0);
        // LATER remove cvars.hud_pause_x/y if raw_canvas removed
        render_text_with_shadow(
            &cvars,
            "PAUSED",
            (view_size.x as f32 - paused_size.width) / 2.0,
            (view_size.y as f32 - paused_size.height) / 2.0,
            cvars.hud_pause_font_size,
            RED,
            cvars.hud_pause_shadow_mq_x,
            cvars.hud_pause_shadow_mq_y,
            1.0,
        );
    }

    // Draw world debug text
    DEBUG_TEXTS_WORLD.with(|texts| {
        let texts = texts.borrow();
        if cvars.d_draw && cvars.d_draw_world_text {
            for text in texts.iter() {
                let scr_pos = text.pos + camera_offset;
                if cull(scr_pos) {
                    // LATER Technically the text can be so long
                    // that it's culled overzealously but meh, perf is more important.
                    continue;
                }

                render_text_with_shadow(
                    &cvars,
                    &text.msg,
                    scr_pos.x as f32,
                    scr_pos.y as f32,
                    16.0,
                    RED,
                    1.0,
                    1.0,
                    0.5,
                );
            }
        }
    });
}

fn render_shared(client: &MacroquadClient, server: &Server, cvars: &Cvars) {
    // Draw screen space debug info:

    let screen_size = Vec2f::new(screen_width() as f64, screen_height() as f64);

    // Draw FPS
    if cvars.d_fps {
        let fps_pos = hud_pos(Vec2f::zero(), screen_size, cvars.d_fps_x, cvars.d_fps_y);
        render_text_with_shadow(
            &cvars,
            &format!(
                "update FPS: {:.1}   gamelogic FPS: {:.1}   render FPS: {:.1}",
                server.update_fps.get_fps(),
                server.gamelogic_fps.get_fps(),
                client.render_fps.get_fps()
            ),
            fps_pos.x - 120.0, // LATER remove the offset after finding a decent font
            fps_pos.y,
            16.0,
            RED,
            1.0,
            1.0,
            0.5,
        );
    }

    // Draw perf info
    if cvars.d_draw && cvars.d_draw_perf {
        render_text_with_shadow(
            &cvars,
            &format!("last {} frames (in ms):", cvars.d_timing_samples),
            screen_size.x as f32 - 280.0,
            screen_size.y as f32 - 105.0,
            16.0,
            RED,
            1.0,
            1.0,
            0.5,
        );
        if let Some((avg, max)) = server.update_durations.get_stats() {
            let text = format!("update avg: {:.1}, max: {:.1}", avg * 1000.0, max * 1000.0);
            render_text_with_shadow(
                &cvars,
                &text,
                screen_size.x as f32 - 280.0,
                screen_size.y as f32 - 90.0,
                16.0,
                RED,
                1.0,
                1.0,
                0.5,
            );
        }
        if let Some((avg, max)) = server.gamelogic_durations.get_stats() {
            let text = format!(
                "gamelogic avg: {:.1}, max: {:.1}",
                avg * 1000.0,
                max * 1000.0
            );
            render_text_with_shadow(
                &cvars,
                &text,
                screen_size.x as f32 - 280.0,
                screen_size.y as f32 - 75.0,
                16.0,
                RED,
                1.0,
                1.0,
                0.5,
            );
        }
        if let Some((avg, max)) = client.render_cmds_durations.get_stats() {
            let text = format!(
                "render cmds avg: {:.1}, max: {:.1}",
                avg * 1000.0,
                max * 1000.0
            );
            render_text_with_shadow(
                &cvars,
                &text,
                screen_size.x as f32 - 280.0,
                screen_size.y as f32 - 60.0,
                16.0,
                RED,
                1.0,
                1.0,
                0.5,
            );
        }
        if let Some((avg, max)) = client.rest_durations.get_stats() {
            let text = format!("rest avg: {:.1}, max: {:.1}", avg * 1000.0, max * 1000.0);
            render_text_with_shadow(
                &cvars,
                &text,
                screen_size.x as f32 - 280.0,
                screen_size.y as f32 - 45.0,
                16.0,
                RED,
                1.0,
                1.0,
                0.5,
            );
        }
    }

    // Draw debug text
    let mut y = 25.0;
    DEBUG_TEXTS.with(|texts| {
        let texts = texts.borrow();
        if cvars.d_draw && cvars.d_draw_text {
            for text in texts.iter() {
                render_text_with_shadow(&cvars, text, 20.0, y as f32, 16.0, RED, 1.0, 1.0, 0.5);
                y += cvars.d_draw_text_line_height;
            }
        }
    });
}

/// Place the image's *center* at `scr_pos`,
/// rotate it clockwise by `angle`.
///
/// See Vec2f for more about the coord system and rotations.
fn render_img_center(img: Texture2D, pos: Vec2f, angle: f64) {
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
fn render_img_offset(img: Texture2D, pos: Vec2f, angle: f64, offset: Vec2f) {
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

fn render_tile(img: Texture2D, x: f64, y: f64, angle: f64) {
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
    shadow_alpha: f64,
) {
    if cvars.r_align_to_pixels_text {
        x = x.floor();
        y = y.floor();
    }
    if shadow_offset_x != 0.0 || shadow_offset_y != 0.0 {
        draw_text(
            &text,
            x + shadow_offset_x,
            y + shadow_offset_y,
            font_size as f32,
            Color::new(0.0, 0.0, 0.0, shadow_alpha as f32),
        );
    }
    draw_text(&text, x, y, font_size as f32, color);
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

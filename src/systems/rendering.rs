//! Rendering to an HTML5 canvas using its 2D API.

use std::{cmp::Reverse, f64::consts::PI};

use js_sys::Array;
use vek::{Clamp, Vec2};
use wasm_bindgen::JsValue;
use web_sys::HtmlImageElement;

use crate::{
    cvars::Cvars,
    debugging::{DEBUG_CROSSES, DEBUG_LINES, DEBUG_TEXTS, DEBUG_TEXTS_WORLD},
    entities::{Ammo, Weapon},
    game_state::Explosion,
    map::F64Ext,
    map::Vec2f,
    map::VecExt,
    map::{Kind, TILE_SIZE},
    Client, Game,
};

/// Redraw the whole canvas.
pub(crate) fn draw(game: &Game, cvars: &Cvars) -> Result<(), JsValue> {
    // This is one long function. A lot of people will tell you that's bad™
    // because they've heard it from other people who think long functions are bad™.
    // Most of those people haven't written a game bigger than snake. Carmack says it's ok so it's ok:
    // http://number-none.com/blow/blog/programming/2014/09/26/carmack-on-inlined-code.html

    let client = &game.client;
    let server = &game.server;
    // No smoothing makes nicer rockets (more like original RW).
    // This also means everything is aligned to pixels
    // without the need to explicitly round x and y in draw calls to whole numbers.
    // TODO revisit when drawing vehicles - maybe make configurable per drawn object
    //      if disabling, try changing quality
    client
        .context
        .set_image_smoothing_enabled(cvars.r_smoothing);

    let player = &server.gs.players[client.player_handle];
    // TODO what if no vehicle
    let player_veh_pos = server.gs.vehicles[player.vehicle.unwrap()].pos;
    let player_entity_pos = if let Some(gm_handle) = player.guided_missile {
        server.gs.projectiles[gm_handle].pos
    } else {
        player_veh_pos
    };

    // Don't put the camera so close to the edge that it would render area outside the map.
    // TODO handle maps smaller than canvas (currently crashes on unreachable)
    let camera_min = client.canvas_size / 2.0;
    let map_size = server.map.maxs();
    let camera_max = map_size - camera_min;
    let camera_pos = player_entity_pos.clamped(camera_min, camera_max);

    // Position of the camera's top left corner in world coords.
    // Subtract this from world coords to get screen coords.
    // Forgetting this is a recurring source of bugs.
    // I've considered making a special type for screen coords (e.g. struct Vec2screen(Vec2f);)
    // so you couldn't accidentally pass world coords to drawing fns but it turned out to be more work than expected:
    // - The newtype had to manually impl all the needed operations of the underlying Vec2 type because ops don't autoderef.
    // - What would be the result of ops that take one world coord and one screen coord? Lots of cases to think about.
    // - Which type are sizes? E.g. `center = corner + size/2` makes sense in both screen and world coords.
    let top_left = camera_pos - camera_min;

    let top_left_tp = server.map.tile_pos(top_left);
    let top_left_index = top_left_tp.index;
    let bg_offset = if cvars.r_align_to_pixels_background {
        top_left_tp.offset.floor()
    } else {
        top_left_tp.offset
    };

    // Draw non-walls
    let mut r = top_left_index.y;
    let mut y = -bg_offset.y;
    while y < client.canvas_size.y {
        let mut c = top_left_index.x;
        let mut x = -bg_offset.x;
        while x < client.canvas_size.x {
            let tile = server.map.col_row(c, r);

            if server.map.surface_of(tile).kind != Kind::Wall {
                let img = &client.imgs_tiles[tile.surface_index];
                draw_tile(client, img, Vec2::new(x, y), tile.angle)?;
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

    // Is the object certainly outside camera view?
    let cull = |scr_pos: Vec2f| {
        // There is no single object bigger than TILE_SIZE (except lines).
        scr_pos.x < -TILE_SIZE
            || scr_pos.y < -TILE_SIZE
            || scr_pos.x > client.canvas_size.x + TILE_SIZE
            || scr_pos.y > client.canvas_size.y + TILE_SIZE
    };

    // Draw MGs
    client.context.set_stroke_style(&"yellow".into());
    for (_, mg) in weapon_projectiles(Weapon::Mg) {
        let scr_pos = mg.pos - top_left;
        if cull(scr_pos) {
            continue;
        }
        client.context.begin_path();
        client.context.move_to(scr_pos.x, scr_pos.y);
        // we're drawing from the bullet's position backwards
        let scr_end = scr_pos - mg.vel.normalized() * cvars.g_machine_gun_trail_length;
        line_to(client, scr_end);
        client.context.stroke();
    }

    // Draw railguns
    client.context.set_stroke_style(&"blue".into());
    for beam in &server.gs.rail_beams {
        let scr_begin = beam.begin - top_left;
        let scr_end = beam.end - top_left;
        client.context.begin_path();
        move_to(client, scr_begin);
        line_to(client, scr_end);
        client.context.stroke();
    }

    // Draw cluster bombs
    if cvars.r_draw_cluster_bombs {
        client.context.set_fill_style(&"rgb(0, 255, 255)".into());
        let shadow_rgba = format!("rgba(0, 0, 0, {})", cvars.g_cluster_bomb_shadow_alpha);
        client.context.set_shadow_color(&shadow_rgba);
        client
            .context
            .set_shadow_offset_x(cvars.g_cluster_bomb_shadow_x);
        client
            .context
            .set_shadow_offset_y(cvars.g_cluster_bomb_shadow_y);
        for (_, cb) in weapon_projectiles(Weapon::Cb) {
            let scr_pos = cb.pos - top_left;
            if cull(scr_pos) {
                continue;
            }
            client.context.fill_rect(
                scr_pos.x - cvars.g_cluster_bomb_size / 2.0,
                scr_pos.y - cvars.g_cluster_bomb_size / 2.0,
                cvars.g_cluster_bomb_size,
                cvars.g_cluster_bomb_size,
            );
        }
        client.context.set_shadow_offset_x(0.0);
        client.context.set_shadow_offset_y(0.0);
    }

    // Draw rockets, homing and guided missiles
    for (_, proj) in weapon_projectiles(Weapon::Rockets) {
        let scr_pos = proj.pos - top_left;
        if cull(scr_pos) {
            continue;
        }
        draw_img_center(client, &client.img_rocket, scr_pos, proj.vel.to_angle())?;
    }
    for (_, proj) in weapon_projectiles(Weapon::Hm) {
        let scr_pos = proj.pos - top_left;
        if cull(scr_pos) {
            continue;
        }
        draw_img_center(client, &client.img_hm, scr_pos, proj.vel.to_angle())?;
    }
    for (_, proj) in weapon_projectiles(Weapon::Gm) {
        let scr_pos = proj.pos - top_left;
        if cull(scr_pos) {
            continue;
        }
        draw_img_center(client, &client.img_gm, scr_pos, proj.vel.to_angle())?;
    }

    // Draw BFGs
    client.context.set_fill_style(&"lime".into());
    client.context.set_stroke_style(&"lime".into());
    for (_, bfg) in weapon_projectiles(Weapon::Bfg) {
        let scr_pos = bfg.pos - top_left;
        if cull(scr_pos) {
            continue;
        }
        client.context.begin_path();
        client
            .context
            .arc(scr_pos.x, scr_pos.y, cvars.g_bfg_radius, 0.0, 2.0 * PI)?;
        client.context.fill();
    }
    for &(src, dest) in &server.gs.bfg_beams {
        let scr_src = src - top_left;
        let scr_dest = dest - top_left;
        client.context.begin_path();
        move_to(client, scr_src);
        line_to(client, scr_dest);
        client.context.stroke();
    }

    // Draw chassis
    for (_, vehicle) in server.gs.vehicles.iter() {
        let scr_pos = vehicle.pos - top_left;
        if cull(scr_pos) {
            continue;
        }
        let img;
        if vehicle.destroyed() {
            img = &client.imgs_wrecks[vehicle.veh_type as usize];
        } else {
            img = &client.imgs_vehicles[vehicle.veh_type as usize * 2];
        }
        draw_img_center(client, img, scr_pos, vehicle.angle)?;
        if cvars.d_draw && cvars.d_draw_hitboxes {
            client.context.set_stroke_style(&"yellow".into());
            client.context.begin_path();
            let corners = vehicle.hitbox.corners(scr_pos, vehicle.angle);
            move_to(client, corners[0]);
            line_to(client, corners[1]);
            line_to(client, corners[2]);
            line_to(client, corners[3]);
            client.context.close_path();
            client.context.stroke();
        }
    }

    // TODO Draw cow

    // Draw turrets
    for (_, vehicle) in server.gs.vehicles.iter() {
        if vehicle.destroyed() {
            continue;
        }

        let scr_pos = vehicle.pos - top_left;
        if cull(scr_pos) {
            continue;
        }

        let img = &client.imgs_vehicles[vehicle.veh_type as usize * 2 + 1];
        let offset_chassis =
            vehicle.angle.to_mat2f() * cvars.g_vehicle_turret_offset_chassis(vehicle.veh_type);
        let turret_scr_pos = scr_pos + offset_chassis;
        let offset_turret = cvars.g_vehicle_turret_offset_turret(vehicle.veh_type);
        draw_img_offset(
            client,
            img,
            turret_scr_pos,
            vehicle.angle + vehicle.turret_angle_current,
            offset_turret,
        )?;
    }

    // Draw explosions
    let iter: Box<dyn Iterator<Item = &Explosion>> = if cvars.r_explosions_reverse {
        Box::new(server.gs.explosions.iter().rev())
    } else {
        Box::new(server.gs.explosions.iter())
    };
    for explosion in iter {
        let scr_pos = explosion.pos - top_left;
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
            img = &client.img_explosion_cyan;
        } else {
            offset = frame * 100.0;
            img = &client.img_explosion;
        };
        client
            .context
            .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                img,
                offset,
                0.0,
                100.0,
                100.0,
                scr_pos.x - 50.0 * explosion.scale,
                scr_pos.y - 50.0 * explosion.scale,
                100.0 * explosion.scale,
                100.0 * explosion.scale,
            )?;
    }

    // Draw walls
    // They are above explosions and turrets, just like in RecWar.
    let mut r = top_left_index.y;
    let mut y = -bg_offset.y;
    while y < client.canvas_size.y {
        let mut c = top_left_index.x;
        let mut x = -bg_offset.x;
        while x < client.canvas_size.x {
            let tile = server.map.col_row(c, r);

            if server.map.surface_of(tile).kind == Kind::Wall {
                let img = &client.imgs_tiles[tile.surface_index];
                draw_tile(client, img, Vec2::new(x, y), tile.angle)?;
            }

            c += 1;
            x += TILE_SIZE;
        }
        r += 1;
        y += TILE_SIZE;
    }

    // Draw world-space HUD elements:

    // Names
    if cvars.hud_names {
        let names_rgba = format!(
            "rgba({0}, {0}, {0}, {1})",
            cvars.hud_names_brightness, cvars.hud_names_alpha
        );
        let shadow_rgba = format!("rgba(0, 0, 0, {})", cvars.hud_names_shadow_alpha);
        client.context.set_shadow_color(&shadow_rgba);
        client.context.set_shadow_offset_x(cvars.hud_names_shadow_x);
        client.context.set_shadow_offset_y(cvars.hud_names_shadow_y);
        client.context.set_fill_style(&names_rgba.into());
        for (_, vehicle) in server.gs.vehicles.iter() {
            let scr_pos = vehicle.pos - top_left;
            if cull(scr_pos) {
                // LATER, restrict name length
                continue;
            }

            let name = &server.gs.players[vehicle.owner].name;
            client.context.fill_text(
                name,
                scr_pos.x + cvars.hud_names_x,
                scr_pos.y + cvars.hud_names_y,
            )?;
        }
        client.context.set_shadow_offset_x(0.0);
        client.context.set_shadow_offset_y(0.0);
    }

    // Homing missile indicator
    let player_veh_scr_pos = player_veh_pos - top_left;
    client.context.set_stroke_style(&"rgb(0, 255, 0)".into());
    let dash_len = cvars.hud_missile_indicator_dash_length.into();
    let dash_pattern = Array::of2(&dash_len, &dash_len);
    client.context.set_line_dash(&dash_pattern)?;
    client.context.begin_path();
    client.context.arc(
        player_veh_scr_pos.x,
        player_veh_scr_pos.y,
        cvars.hud_missile_indicator_radius,
        0.0,
        2.0 * PI,
    )?;
    move_to(client, player_veh_scr_pos);
    //let dir = (game.gs.gm.pos - player_veh_pos.0).normalized();
    let dir = 0.0.to_vec2f(); // TODO
    let end = player_veh_scr_pos + dir * cvars.hud_missile_indicator_radius;
    line_to(client, end);
    client.context.stroke();
    client.context.set_line_dash(&Array::new())?;

    // Debug lines and crosses
    DEBUG_LINES.with(|lines| {
        let mut lines = lines.borrow_mut();
        for line in lines.iter_mut() {
            if cvars.d_draw && cvars.d_draw_lines {
                client.context.set_stroke_style(&line.color.into());

                let scr_begin = line.begin - top_left;
                let scr_end = line.end - top_left;
                client.context.begin_path();
                move_to(client, scr_begin);
                line_to(client, scr_end);
                if cvars.d_draw_lines_ends_length > 0.0 {
                    let segment = line.end - line.begin;
                    let perpendicular = Vec2f::new(-segment.y, segment.x).normalized();
                    move_to(
                        client,
                        scr_begin + -perpendicular * cvars.d_draw_lines_ends_length,
                    );
                    line_to(
                        client,
                        scr_begin + perpendicular * cvars.d_draw_lines_ends_length,
                    );
                    move_to(
                        client,
                        scr_end + -perpendicular * cvars.d_draw_lines_ends_length,
                    );
                    line_to(
                        client,
                        scr_end + perpendicular * cvars.d_draw_lines_ends_length,
                    );
                }
                client.context.stroke();
            }
            line.time -= server.gs.dt;
        }
    });
    DEBUG_CROSSES.with(|crosses| {
        let mut crosses = crosses.borrow_mut();
        for cross in crosses.iter_mut() {
            if cvars.d_draw && cvars.d_draw_crosses {
                let scr_point = cross.point - top_left;
                if cull(scr_point) {
                    continue;
                }

                client.context.set_stroke_style(&cross.color.into());
                let top_left = scr_point - Vec2f::new(-3.0, -3.0);
                let bottom_right = scr_point - Vec2f::new(3.0, 3.0);
                let top_right = scr_point - Vec2f::new(3.0, -3.0);
                let bottom_left = scr_point - Vec2f::new(-3.0, 3.0);
                client.context.begin_path();
                move_to(client, top_left);
                line_to(client, bottom_right);
                move_to(client, top_right);
                line_to(client, bottom_left);
                client.context.stroke();
            }
            cross.time -= server.gs.dt;
        }
    });

    // Draw screen-space HUD elements:

    let mut player_points: Vec<_> = server
        .gs
        .players
        .iter()
        .map(|(index, player)| {
            let points = player.score.kills * cvars.g_ffa_score_kill
                + player.score.deaths * cvars.g_ffa_score_death;
            (index, points)
        })
        .collect();
    player_points.sort_by_key(|&(_, points)| Reverse(points));

    // Score
    // Original RW shows current score as a big bold number with a 2px shadow.
    client.context.set_shadow_offset_x(cvars.hud_score_shadow_x);
    client.context.set_shadow_offset_y(cvars.hud_score_shadow_y);
    let score_font = format!("{}px sans-serif", cvars.hud_score_font_size);
    client.context.set_font(&score_font);
    let score_pos = hud_pos(client, cvars.hud_score_x, cvars.hud_score_y);
    client.context.fill_text(
        &(player.score.kills - player.score.deaths).to_string(),
        score_pos.x,
        score_pos.y,
    )?;

    // Ranking
    // Original RW shows "current rank / total players (+/- points difference to leader or second)"
    // as a big but not bold number with a 1px shadow. E.g. "1/3 (+5)" or "2/3 (0)".
    // There's no special treatement for players with the same number of points.
    client
        .context
        .set_shadow_offset_x(cvars.hud_ranking_shadow_x);
    client
        .context
        .set_shadow_offset_y(cvars.hud_ranking_shadow_y);
    let ranking_font = format!("{}px sans-serif", cvars.hud_ranking_font_size);
    client.context.set_font(&ranking_font);
    let ranking_pos = hud_pos(client, cvars.hud_ranking_x, cvars.hud_ranking_y);
    let current_index = player_points
        .iter()
        .position(|&(handle, _)| handle == client.player_handle)
        .unwrap();
    let points_diff = if current_index == 0 {
        if player_points.len() == 2 {
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
    client
        .context
        .fill_text(&ranking, ranking_pos.x, ranking_pos.y)?;

    client.context.set_font("10px sans-serif");
    client.context.set_shadow_offset_x(0.0);
    client.context.set_shadow_offset_y(0.0);

    // Hit points (goes from green to red)
    // Might wanna use https://crates.io/crates/colorsys if I need more color operations.
    // Hit points to color (poor man's HSV):
    // 0.0 = red
    // 0.0..0.5 -> increase green channel
    // 0.5 = yellow
    // 0.5..1.0 -> decrease red channel
    // 1.0 = green
    let player_vehicle = &server.gs.vehicles[player.vehicle.unwrap()]; // TODO what if no vehicle
    let r = 1.0 - (player_vehicle.hp_fraction.clamped(0.5, 1.0) - 0.5) * 2.0;
    let g = player_vehicle.hp_fraction.clamped(0.0, 0.5) * 2.0;
    let rgb = format!("rgb({}, {}, 0)", r * 255.0, g * 255.0);
    client.context.set_fill_style(&rgb.into());
    let hp_pos = hud_pos(client, cvars.hud_hp_x, cvars.hud_hp_y);
    client.context.fill_rect(
        hp_pos.x,
        hp_pos.y,
        cvars.hud_hp_width * player_vehicle.hp_fraction,
        cvars.hud_hp_height,
    );
    if cvars.d_draw_text {
        client.context.set_fill_style(&"red".into());
        let hp_number = player_vehicle.hp_fraction * cvars.g_vehicle_hp(player_vehicle.veh_type);
        let hp_text = format!("{}", hp_number);
        client
            .context
            .fill_text(&hp_text, hp_pos.x - 25.0, hp_pos.y + cvars.hud_hp_height)?;
    }

    // Ammo
    client.context.set_fill_style(&"yellow".into());
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
    let ammo_pos = hud_pos(client, cvars.hud_ammo_x, cvars.hud_ammo_y);
    client.context.fill_rect(
        ammo_pos.x,
        ammo_pos.y,
        cvars.hud_ammo_width * ammo_fraction,
        cvars.hud_ammo_height,
    );
    if cvars.d_draw_text {
        client.context.set_fill_style(&"red".into());
        let ammo_number = match ammo {
            Ammo::Loaded(_ready_time, count) => count,
            Ammo::Reloading(_start, _end) => 0,
        };
        client.context.fill_text(
            &ammo_number.to_string(),
            ammo_pos.x - 25.0,
            ammo_pos.y + cvars.hud_ammo_height,
        )?;
    }

    // Weapon icon
    // The original shadows were part of the image but this is good enough for now.
    let shadow_rgba = format!("rgba(0, 0, 0, {})", cvars.hud_weapon_icon_shadow_alpha);
    client.context.set_shadow_color(&shadow_rgba);
    client
        .context
        .set_shadow_offset_x(cvars.hud_weapon_icon_shadow_x);
    client
        .context
        .set_shadow_offset_y(cvars.hud_weapon_icon_shadow_y);
    draw_img_center(
        client,
        &client.imgs_weapon_icons[player.cur_weapon as usize],
        hud_pos(client, cvars.hud_weapon_icon_x, cvars.hud_weapon_icon_y),
        0.0,
    )?;
    client.context.set_shadow_offset_x(0.0);
    client.context.set_shadow_offset_y(0.0);

    // Scoreboard
    if player_vehicle.destroyed() {
        client
            .context
            .set_shadow_offset_x(cvars.hud_scoreboard_shadow_x);
        client
            .context
            .set_shadow_offset_y(cvars.hud_scoreboard_shadow_y);
        client.context.set_fill_style(&"white".into());

        let height = (server.gs.players.len() + 1) as f64 * cvars.hud_scoreboard_line_height;
        let mut y = (client.canvas_size.y - height) / 2.0;
        let x = (client.canvas_size.x - 200.0) / 2.0;

        let header_font = format!("bold {}px sans-serif", cvars.hud_scoreboard_font_size);
        client.context.set_font(&header_font);
        client.context.fill_text("Name", x, y)?;
        client.context.fill_text("Kills", x + 150.0, y)?;
        client.context.fill_text("Deaths", x + 200.0, y)?;
        client.context.fill_text("Points", x + 270.0, y)?;
        y += cvars.hud_scoreboard_line_height;

        let entry_font = format!("{}px sans-serif", cvars.hud_scoreboard_font_size);
        client.context.set_font(&entry_font);
        for (player_handle, points) in player_points {
            let player = &server.gs.players[player_handle];
            client.context.fill_text(&player.name, x, y)?;
            client
                .context
                .fill_text(&player.score.kills.to_string(), x + 150.0, y)?;
            client
                .context
                .fill_text(&player.score.deaths.to_string(), x + 200.0, y)?;
            client
                .context
                .fill_text(&points.to_string(), x + 270.0, y)?;
            y += cvars.hud_scoreboard_line_height;
        }
        client.context.set_font("10px sans-serif");
        client.context.set_shadow_offset_x(0.0);
        client.context.set_shadow_offset_y(0.0);
    }

    // Pause
    if server.paused {
        client.context.set_shadow_offset_x(cvars.hud_pause_shadow_x);
        client.context.set_shadow_offset_y(cvars.hud_pause_shadow_y);
        client.context.set_fill_style(&"red".into());
        let pause_font = format!("{}px sans-serif", cvars.hud_pause_font_size);
        client.context.set_font(&pause_font);

        client
            .context
            .fill_text("PAUSED", cvars.hud_pause_x, cvars.hud_pause_y)?;

        client.context.set_font("10px sans-serif");
        client.context.set_shadow_offset_x(0.0);
        client.context.set_shadow_offset_y(0.0);
    }

    // Draw screen space debug info:
    client.context.set_fill_style(&"red".into());

    // Draw FPS
    if cvars.d_fps {
        let fps = client.fps.get_fps();
        client.context.fill_text(
            &format!("FPS: {:.1}", fps),
            client.canvas_size.x - 60.0,
            client.canvas_size.y - 15.0,
        )?;
    }

    // Draw perf info
    if cvars.d_draw && cvars.d_draw_perf {
        client.context.fill_text(
            &format!("last {} frames:", cvars.d_timing_samples),
            client.canvas_size.x - 150.0,
            client.canvas_size.y - 75.0,
        )?;
        if let Some((avg, max)) = game.update_durations.get_stats() {
            client.context.fill_text(
                &format!("update avg: {:.1}, max: {:.1}", avg, max),
                client.canvas_size.x - 150.0,
                client.canvas_size.y - 60.0,
            )?;
        }
        if let Some((avg, max)) = game.draw_durations.get_stats() {
            client.context.fill_text(
                &format!("draw avg: {:.1}, max: {:.1}", avg, max),
                client.canvas_size.x - 150.0,
                client.canvas_size.y - 45.0,
            )?;
        }
    }

    // Draw world debug text
    DEBUG_TEXTS_WORLD.with(|texts| {
        let texts = texts.borrow();
        if cvars.d_draw && cvars.d_draw_world_text {
            for text in texts.iter() {
                let scr_pos = text.pos - top_left;
                if cull(scr_pos) {
                    // Technically the text can be so long
                    // that it's culled overzealously but meh, perf is more important.
                    continue;
                }

                client
                    .context
                    .fill_text(&text.msg, scr_pos.x, scr_pos.y)
                    .unwrap();
            }
        }
    });

    // Draw debug text
    let mut y = 25.0;
    DEBUG_TEXTS.with(|texts| {
        let texts = texts.borrow();
        if cvars.d_draw && cvars.d_draw_text {
            for text in texts.iter() {
                client.context.fill_text(text, 20.0, y).unwrap();
                y += cvars.d_draw_text_line_height;
            }
        }
    });

    Ok(())
}

fn move_to(client: &Client, point: Vec2f) {
    client.context.move_to(point.x, point.y);
}

fn line_to(client: &Client, point: Vec2f) {
    client.context.line_to(point.x, point.y);
}

/// Place the `tile`'s *top-left corner* at `scr_pos`,
/// rotate it clockwise around its center.
fn draw_tile(
    client: &Client,
    tile: &HtmlImageElement,
    scr_pos: Vec2f,
    angle: f64,
) -> Result<(), JsValue> {
    draw_img_offset(
        client,
        tile,
        scr_pos + TILE_SIZE / 2.0,
        angle,
        Vec2f::zero(),
    )
}

/// Place the image's *center* at `scr_pos`,
/// rotate it clockwise by `angle`.
///
/// See Vec2f for more about the coord system and rotations.
fn draw_img_center(
    client: &Client,
    img: &HtmlImageElement,
    scr_pos: Vec2f,
    angle: f64,
) -> Result<(), JsValue> {
    draw_img_offset(client, img, scr_pos, angle, Vec2f::zero())
}

/// Place the `img`'s *center of rotation* at `scr_pos`,
/// rotate it clockwise by `angle`.
/// The center of rotation is `img`'s center + `offset`.
///
/// See Vec2f for more about the coord system and rotations.
fn draw_img_offset(
    client: &Client,
    img: &HtmlImageElement,
    scr_pos: Vec2f,
    angle: f64,
    offset: Vec2f,
) -> Result<(), JsValue> {
    let half_size = Vec2::new(img.natural_width(), img.natural_height()).as_() / 2.0;
    let offset = offset + half_size;
    client.context.translate(scr_pos.x, scr_pos.y)?;
    client.context.rotate(angle)?;
    // This is the same as translating by -offset, then drawing at 0,0.
    client
        .context
        .draw_image_with_html_image_element(img, -offset.x, -offset.y)?;
    client.context.reset_transform()?;
    Ok(())
}

/// If x or y are negative, count them from the right or bottom respectively.
/// Useful to make HUD config cvars work for any canvas size.
fn hud_pos(client: &Client, mut x: f64, mut y: f64) -> Vec2f {
    if x < 0.0 {
        x += client.canvas_size.x;
    }
    if y < 0.0 {
        y += client.canvas_size.y;
    }
    Vec2f::new(x, y)
}

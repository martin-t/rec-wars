//! Native and WASM versions using the macroquad engine.

mod rendering;

use thunderdome::Index;

use macroquad::prelude::*;

use rec_wars::{
    cvars::Cvars,
    game_state::Input,
    map::Vec2f,
    server::Server,
    timing::{Durations, Fps},
};

#[derive(Debug)]
pub(crate) struct MacroquadClient {
    pub(crate) imgs_tiles: Vec<Texture2D>,
    pub(crate) imgs_vehicles: Vec<Texture2D>,
    pub(crate) imgs_wrecks: Vec<Texture2D>,
    pub(crate) imgs_weapon_icons: Vec<Texture2D>,
    pub(crate) img_rocket: Texture2D,
    pub(crate) img_hm: Texture2D,
    pub(crate) img_gm: Texture2D,
    pub(crate) img_explosion: Texture2D,
    pub(crate) img_explosion_cyan: Texture2D,
    pub(crate) render_fps: Fps,
    pub(crate) render_cmds_durations: Durations,
    pub(crate) rest_durations: Durations,
    pub(crate) viewport_size: Vec2f,
    pub(crate) render_targets: Option<(RenderTarget, RenderTarget)>,
    pub(crate) player_handle: Index,
}

impl MacroquadClient {
    pub(crate) async fn new(cvars: &Cvars, player1_handle: Index) -> Self {
        // TODO load all in parallel

        let mut imgs_tiles = Vec::new();
        for path in &[
            "assets/tiles/g1.bmp",
            "assets/tiles/g2.bmp",
            "assets/tiles/g3.bmp",
            "assets/tiles/g_stripes.bmp",
            "assets/tiles/bunker1.bmp",
            "assets/tiles/ice1.bmp",
            "assets/tiles/ice.bmp",
            "assets/tiles/ice_side.bmp",
            "assets/tiles/ice_corner.bmp",
            "assets/tiles/g_spawn.bmp",
            "assets/tiles/road.bmp",
            "assets/tiles/water.bmp",
            "assets/tiles/snow.bmp",
            "assets/tiles/snow2.bmp",
            "assets/tiles/bunker2.bmp",
            "assets/tiles/base.bmp",
            "assets/tiles/water_side.bmp",
            "assets/tiles/water_corner.bmp",
            "assets/tiles/desert.bmp",
            "assets/tiles/d_rock.bmp",
            "assets/tiles/g2d.bmp",
            "assets/tiles/water_middle.bmp",
        ] {
            imgs_tiles.push(load_texture(path).await.unwrap());
        }

        let mut imgs_vehicles = Vec::new();
        for path in &[
            "assets/vehicles/tank_chassis_flames.png",
            "assets/vehicles/tank_turret_flames.png",
            "assets/vehicles/hovercraft_chassis_flames.png",
            "assets/vehicles/hovercraft_turret_flames.png",
            "assets/vehicles/hummer_chassis_flames.png",
            "assets/vehicles/hummer_turret_flames.png",
        ] {
            imgs_vehicles.push(load_texture(path).await.unwrap());
        }

        let mut imgs_wrecks = Vec::new();
        for path in &[
            "assets/wrecks/tank.png",
            "assets/wrecks/hovercraft.png",
            "assets/wrecks/hummer.png",
        ] {
            imgs_wrecks.push(load_texture(path).await.unwrap());
        }

        let mut imgs_weapon_icons = Vec::new();
        for path in &[
            "assets/weapon_icons/mg.png",
            "assets/weapon_icons/rail.png",
            "assets/weapon_icons/cb.png",
            "assets/weapon_icons/rockets.png",
            "assets/weapon_icons/hm.png",
            "assets/weapon_icons/gm.png",
            "assets/weapon_icons/bfg.png",
        ] {
            imgs_weapon_icons.push(load_texture(path).await.unwrap());
        }

        // LATER smoothing optional and configurable per image
        // LATER either use or remove r_smoothing (if raw_canvas is removed)
        let img_rocket = load_texture("assets/weapons/rocket.png").await.unwrap();
        let img_hm = load_texture("assets/weapons/hm.png").await.unwrap();
        let img_gm = load_texture("assets/weapons/gm.png").await.unwrap();
        let img_explosion = load_texture("assets/explosion.png").await.unwrap();
        img_explosion.set_filter(FilterMode::Nearest);
        let img_explosion_cyan = load_texture("assets/explosion_cyan.png").await.unwrap();
        img_explosion_cyan.set_filter(FilterMode::Nearest);

        println!(
            "Detected screen size: {}x{}",
            screen_width(),
            screen_height()
        );
        let viewport_width = (screen_width() as f64 - cvars.r_splitscreen_gap) / 2.0;
        let viewport_size = Vec2f::new(viewport_width, screen_height() as f64);
        let viewport_left = render_target(viewport_size.x as u32, viewport_size.y as u32);
        let viewport_right = render_target(viewport_size.x as u32, viewport_size.y as u32);

        Self {
            imgs_tiles,
            imgs_vehicles,
            imgs_wrecks,
            imgs_weapon_icons,
            img_rocket,
            img_hm,
            img_gm,
            img_explosion,
            img_explosion_cyan,
            render_fps: Fps::new(),
            render_cmds_durations: Durations::new(),
            rest_durations: Durations::new(),
            viewport_size,
            render_targets: Some((viewport_left, viewport_right)),
            player_handle: player1_handle,
        }
    }

    pub(crate) fn render(&mut self, server: &Server, cvars: &Cvars) {
        self.render_fps.tick(cvars.d_fps_period, server.real_time);
        let start = get_time();

        rendering::render(&self, &server, &cvars);

        let end = get_time();
        self.render_cmds_durations
            .add(cvars.d_timing_samples, end - start);
    }
}

pub(crate) fn get_input() -> Input {
    let mut input = Input::new();
    if was_input_pressed(&[KeyCode::Left, KeyCode::A]) {
        input.left = true;
    }
    if was_input_pressed(&[KeyCode::Right, KeyCode::D]) {
        input.right = true;
    }
    if was_input_pressed(&[KeyCode::Up, KeyCode::W]) {
        input.up = true;
    }
    if was_input_pressed(&[KeyCode::Down, KeyCode::S]) {
        input.down = true;
    }
    if was_input_pressed(&[KeyCode::Q, KeyCode::N]) {
        input.turret_left = true;
    }
    if was_input_pressed(&[KeyCode::E, KeyCode::M]) {
        input.turret_right = true;
    }
    if was_input_pressed(&[KeyCode::V, KeyCode::Period]) {
        input.prev_weapon = true;
    }
    if was_input_pressed(&[
        KeyCode::LeftShift,
        KeyCode::RightShift,
        KeyCode::B,
        KeyCode::Comma,
    ]) {
        input.next_weapon = true;
    }
    if was_input_pressed(&[KeyCode::Space]) {
        input.fire = true;
    }
    if was_input_pressed(&[KeyCode::J, KeyCode::X]) {
        input.mine = true;
    }
    if was_input_pressed(&[KeyCode::L]) {
        input.self_destruct = true;
    }
    if was_input_pressed(&[KeyCode::H]) {
        input.horn = true;
    }
    if was_input_pressed(&[]) {
        input.chat = true;
    }
    if was_input_pressed(&[KeyCode::Pause, KeyCode::P]) {
        input.pause = true;
    }
    input
}

fn was_input_pressed(key_codes: &[KeyCode]) -> bool {
    for &key_code in key_codes {
        // Check both to avoid skipping input if it's pressed and released within one frame.
        if is_key_pressed(key_code) || is_key_down(key_code) {
            return true;
        }
    }
    false
}

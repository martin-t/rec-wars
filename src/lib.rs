// TODO lints

use std::f64::consts::PI;

use vek::ops::Clamp;
use vek::Vec2;

use js_sys::Array;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::{CanvasRenderingContext2d, HtmlImageElement};

mod data;

// TODO newtypes with Derefs? tile vs world pos vs screen pos
type Vec2f = Vec2<f64>;
type Vec2u = Vec2<usize>;

type Map = Vec<Vec<usize>>;

const TILE_SIZE: f64 = 64.0;

#[wasm_bindgen]
pub struct World {
    context: CanvasRenderingContext2d,
    canvas_size: Vec2f,
    tiles: Vec<HtmlImageElement>,
    map: Map,
    prev_update: f64,
    pos: Vec2f,
    vel: Vec2f,
    explosions: Vec<(Vec2f, i32)>,
    debug_texts: Vec<String>,
}

#[wasm_bindgen]
impl World {
    #[wasm_bindgen(constructor)]
    pub fn new(
        context: CanvasRenderingContext2d,
        width: f64,
        height: f64,
        tiles: Array,
        map_text: &str,
    ) -> Self {
        console_error_panic_hook::set_once();

        let tiles = tiles.iter().map(|tile| tile.dyn_into().unwrap()).collect();
        let map = data::load_map(map_text);
        Self {
            context,
            canvas_size: Vec2f::new(width, height),
            tiles,
            map,
            prev_update: 0.0,
            pos: Vec2f::new(640.0, 640.0),
            vel: Vec2f::new(0.02, 0.01),
            explosions: Vec::new(),
            debug_texts: Vec::new(),
        }
    }

    pub fn input(&mut self, left: f64, right: f64, up: f64, down: f64) {
        self.vel.x -= left * 0.01;
        self.vel.x += right * 0.01;
        self.vel.y -= up * 0.01;
        self.vel.y += down * 0.01;
    }

    pub fn update_pre(&mut self, t: f64) {
        let dt = t - self.prev_update;

        self.pos += self.vel * dt;
        if self.pos.x <= 0.0 {
            self.pos.x = 0.0;
            self.vel.x = 0.0;
        }
        if self.pos.y <= 0.0 {
            self.pos.y = 0.0;
            self.vel.y = 0.0;
        }
        let map_size = self.map_size();
        if self.pos.x >= map_size.x {
            self.pos.x = map_size.x;
            self.vel.x = 0.0;
        }
        if self.pos.y >= map_size.y {
            self.pos.y = map_size.y;
            self.vel.y = 0.0;
        }

        let tile_pos = Self::to_tile(self.pos);
        let texture = self.map[tile_pos.x][tile_pos.y] / 4;

        // FIXME
        self.debug_text(format!("tex {}", texture));
        if texture == 4 || texture == 14 {
            self.explosions.push((self.pos, 0));
            self.pos = Vec2f::new(640.0, 640.0);
        }

        self.prev_update = t;
    }

    fn to_tile(pos: Vec2f) -> Vec2u {
        // FIXME clamp to bounds?
        let tmp = pos / TILE_SIZE;
        Vec2u::new(tmp.x as usize, tmp.y as usize)
    }

    pub fn draw(
        &mut self,
        img_explosion: &HtmlImageElement,
        img_guided_missile: &HtmlImageElement,
        align_to_pixels: bool,
    ) -> Result<(), JsValue> {
        // Don't put the camera so close to the edge that it would render area outside the map.
        // TODO handle maps smaller than canvas (currently crashes on unreachable)
        assert!(self.map.len() >= 20);
        assert!(self.map[0].len() >= 20);
        // TODO print trace on unreachable?
        let camera_min = self.canvas_size / 2.0;
        let map_size = self.map_size();
        let camera_max = map_size - camera_min;
        let camera_pos = self.pos.clamped(camera_min, camera_max);

        // Draw background
        // This only works properly with positive numbers but it's ok since top left of the map is (0.0, 0.0).
        let top_left = camera_pos - camera_min;
        let top_left_tile = Self::to_tile(top_left);
        let mut offset_in_tile = top_left % TILE_SIZE;
        // TODO align player? other?
        if align_to_pixels {
            offset_in_tile = offset_in_tile.floor();
        }

        let mut c = top_left_tile.x;
        let mut x = -offset_in_tile.x;
        while x < self.canvas_size.x {
            let mut r = top_left_tile.y;
            let mut y = -offset_in_tile.y;
            while y < self.canvas_size.y {
                let index = self.map[r][c] / 4;
                let img = &self.tiles[index];
                let rotation = self.map[r][c] % 4;

                // rotate counterclockwise around tile center
                self.context
                    .translate(x + TILE_SIZE / 2.0, y + TILE_SIZE / 2.0)?;
                self.context.rotate(rotation as f64 * -PI / 2.0)?;
                self.context.translate(-TILE_SIZE / 2.0, -TILE_SIZE / 2.0)?;

                self.context
                    .draw_image_with_html_image_element(img, 0.0, 0.0)?;

                self.context.reset_transform()?;

                r += 1;
                y += TILE_SIZE;
            }
            c += 1;
            x += TILE_SIZE;
        }

        // Draw player
        let player_scr_pos = self.pos - top_left;
        self.context.draw_image_with_html_image_element(
            img_guided_missile,
            player_scr_pos.x - 10.0,
            player_scr_pos.y - 2.0,
        )?;

        // Draw explosions
        // TODO CB explosions happen on walls, just invisible
        for &(pos, frame) in &self.explosions {
            // TODO frame rate independence
            let real_frame = frame / 2; // the sprite is made for 30 fps
            let offset = real_frame as f64 * 100.0;
            let scr_pos = pos - top_left;
            self.context
                .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    img_explosion,
                    offset,
                    0.0,
                    100.0,
                    100.0,
                    scr_pos.x - 50.0,
                    scr_pos.y - 50.0,
                    100.0,
                    100.0,
                )?;
        }

        // Draw debug text
        self.context.set_fill_style(&"red".into());
        let mut y = 20.0;
        for line in &self.debug_texts {
            self.context.fill_text(line, 20.0, y)?;
            y += 10.0;
        }
        self.debug_texts.clear();

        Ok(())
    }

    pub fn update_post(&mut self, t: f64) {
        for explosion in &mut self.explosions {
            explosion.1 += 1;
        }
        self.explosions.retain(|expl| expl.1 < 26);
    }

    #[allow(unused)]
    fn debug_text<S: Into<String>>(&mut self, s: S) {
        self.debug_texts.push(s.into());
    }

    fn map_size(&self) -> Vec2f {
        Vec2f::new(
            self.map.len() as f64 * TILE_SIZE,
            self.map[0].len() as f64 * TILE_SIZE,
        )
    }
}

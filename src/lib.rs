// TODO lints

mod data;

use std::f64::consts::PI;

use js_sys::Array;

use vek::ops::Clamp;
use vek::Vec2;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::{CanvasRenderingContext2d, HtmlImageElement};

use data::{Kind, Map, Surface, Vec2f, TILE_SIZE};

#[wasm_bindgen]
pub struct World {
    context: CanvasRenderingContext2d,
    canvas_size: Vec2f,
    imgs_textures: Vec<HtmlImageElement>,
    img_explosion: HtmlImageElement,
    surfaces: Vec<Surface>,
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
        textures: Array,
        img_explosion: HtmlImageElement,
        textures_text: &str,
        map_text: &str,
    ) -> Self {
        console_error_panic_hook::set_once();

        let imgs_textures = textures
            .iter()
            .map(|tile| tile.dyn_into().unwrap())
            .collect();
        let surfaces = data::load_textures(textures_text);
        let map = data::load_map(map_text);
        Self {
            context,
            canvas_size: Vec2f::new(width, height),
            imgs_textures,
            img_explosion,
            surfaces,
            map,
            prev_update: 0.0,
            pos: Vec2f::new(640.0, 640.0),
            vel: Vec2f::new(0.3, 0.2),
            explosions: Vec::new(),
            debug_texts: Vec::new(),
        }
    }

    pub fn input(&mut self, left: f64, right: f64, up: f64, down: f64) {
        // self.vel.x -= left * 0.01;
        // self.vel.x += right * 0.01;
        // self.vel.y -= up * 0.01;
        // self.vel.y += down * 0.01;

        let accell = 1.0 + up * 0.05 - down * 0.05;
        self.vel *= accell;
        let angle: f64 = right * 2.5 - left * 2.5;
        self.vel.rotate_z(angle.to_radians());
    }

    pub fn update_pre(&mut self, t: f64) {
        let dt = t - self.prev_update;
        // TODO this is broken when minimized (collision detection, etc.)

        self.pos += self.vel * dt;
        if self.pos.x <= 0.0 {
            self.pos.x = 0.0;
            self.vel.x = 0.0;
        }
        if self.pos.y <= 0.0 {
            self.pos.y = 0.0;
            self.vel.y = 0.0;
        }
        let map_size = self.map.maxs();
        if self.pos.x >= map_size.x {
            self.pos.x = map_size.x;
            self.vel.x = 0.0;
        }
        if self.pos.y >= map_size.y {
            self.pos.y = map_size.y;
            self.vel.y = 0.0;
        }

        let tile_pos = self.map.tile_pos(self.pos);
        let surface = self.map[tile_pos.index].surface;
        let kind = self.surfaces[surface].kind;
        if kind == Kind::Wall {
            self.explosions.push((self.pos, 0));
            self.pos = Vec2f::new(640.0, 640.0);
        }

        self.prev_update = t;
    }

    pub fn draw(
        &mut self,
        img_guided_missile: &HtmlImageElement,
        align_to_pixels: bool,
    ) -> Result<(), JsValue> {
        // Don't put the camera so close to the edge that it would render area outside the map.
        // TODO handle maps smaller than canvas (currently crashes on unreachable)
        let camera_min = self.canvas_size / 2.0;
        let map_size = self.map.maxs();
        let camera_max = map_size - camera_min;
        let camera_pos = self.pos.clamped(camera_min, camera_max);

        let top_left = camera_pos - camera_min;
        let top_left_tp = self.map.tile_pos(top_left);
        let top_left_index = top_left_tp.index;
        let bg_offset = if align_to_pixels {
            top_left_tp.offset.floor()
        } else {
            top_left_tp.offset
        };
        // TODO align player? other?

        // Draw non-walls
        let mut r = top_left_index.y;
        let mut y = -bg_offset.y;
        while y < self.canvas_size.y {
            let mut c = top_left_index.x;
            let mut x = -bg_offset.x;
            while x < self.canvas_size.x {
                let tile = self.map.col_row(c, r);

                if self.surfaces[tile.surface].kind != Kind::Wall {
                    let img = &self.imgs_textures[tile.surface];
                    self.draw_img(img, Vec2::new(x, y), tile.rotation)?;
                }

                c += 1;
                x += TILE_SIZE;
            }
            r += 1;
            y += TILE_SIZE;
        }

        // Draw missile
        let player_scr_pos = self.pos - top_left;
        let angle = self.vel.y.atan2(self.vel.x) - PI;
        self.draw_img(img_guided_missile, player_scr_pos, angle)?;

        // Draw explosions
        // TODO CB explosions happen on walls, just partially obscured
        for &(pos, frame) in &self.explosions {
            // TODO frame rate independence
            let real_frame = frame / 2; // the sprite is made for 30 fps
            let offset = real_frame as f64 * 100.0;
            let scr_pos = pos - top_left;
            self.context
                .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    &self.img_explosion,
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

        // Draw walls
        let mut r = top_left_index.y;
        let mut y = -bg_offset.y;
        while y < self.canvas_size.y {
            let mut c = top_left_index.x;
            let mut x = -bg_offset.x;
            while x < self.canvas_size.x {
                let tile = self.map.col_row(c, r);

                if self.surfaces[tile.surface].kind == Kind::Wall {
                    let img = &self.imgs_textures[tile.surface];
                    self.draw_img(img, Vec2::new(x, y), tile.rotation)?;
                }

                c += 1;
                x += TILE_SIZE;
            }
            r += 1;
            y += TILE_SIZE;
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

    pub fn update_post(&mut self) {
        for explosion in &mut self.explosions {
            explosion.1 += 1;
        }
        self.explosions.retain(|expl| expl.1 < 26);
    }

    fn draw_img(&self, img: &HtmlImageElement, screen_pos: Vec2f, rot: f64) -> Result<(), JsValue> {
        // Rotate counterclockwise around img center.
        let half_size = Vec2::new(img.natural_width(), img.natural_height()).as_() / 2.0;
        self.context
            .translate(screen_pos.x + half_size.x, screen_pos.y + half_size.y)?;
        self.context.rotate(rot)?;

        // Now back off to the img's corner and draw it.
        // This can be done either by translating -half_size, then drawing at 0,0
        // or at once by drawing at -half_size which his perhaps marginally more efficient.
        self.context
            .draw_image_with_html_image_element(img, -half_size.x, -half_size.y)?;

        self.context.reset_transform()?;
        Ok(())
    }

    // fn draw_sprite(
    //     &self,
    //     img: &HtmlImageElement,
    //     world_pos: Vec2f,
    //     rot: f64,
    //     source_pos: Vec2f,
    //     source_size: Vec2f,
    // ) -> Result<(), JsValue> {
    //     Ok(())
    // }

    #[allow(unused)]
    fn debug_text<S: Into<String>>(&mut self, s: S) {
        self.debug_texts.push(s.into());
    }
}

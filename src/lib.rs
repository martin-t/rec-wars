// TODO lints

mod cvars;
mod data;
mod entities;
mod logging;

use std::f64::consts::PI;

use js_sys::Array;

use rand::prelude::*;

use vek::ops::Clamp;
use vek::Vec2;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::{CanvasRenderingContext2d, HtmlImageElement};

use cvars::Cvars;
use data::{Kind, Map, Surface, Vec2f, TILE_SIZE};
use entities::GuidedMissile;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct World {
    rng: SmallRng,
    context: CanvasRenderingContext2d,
    canvas_size: Vec2f,
    imgs_textures: Vec<HtmlImageElement>,
    img_guided_missile: HtmlImageElement,
    img_explosion: HtmlImageElement,
    surfaces: Vec<Surface>,
    map: Map,
    prev_update: f64,
    guided_missile: GuidedMissile,
    explosions: Vec<(Vec2f, i32)>,
    debug_texts: Vec<String>,
}

#[wasm_bindgen]
impl World {
    #[wasm_bindgen(constructor)]
    pub fn new(
        cvars: &Cvars,
        context: CanvasRenderingContext2d,
        width: f64,
        height: f64,
        textures: Array,
        img_guided_missile: HtmlImageElement,
        img_explosion: HtmlImageElement,
        tex_list_text: &str,
        map_text: &str,
    ) -> Self {
        console_error_panic_hook::set_once();

        let mut rng = SmallRng::seed_from_u64(5);

        // TODO try https://rustwasm.github.io/docs/wasm-bindgen/reference/types/boxed-jsvalue-slice.html
        let imgs_textures = textures
            .iter()
            .map(|tile| tile.dyn_into().unwrap())
            .collect();

        let surfaces = data::load_tex_list(tex_list_text);
        let map = data::load_map(map_text, &surfaces);
        let guided_missile = entities::spawn_guided_missile(cvars, &mut rng, &map);
        Self {
            rng,
            context,
            canvas_size: Vec2f::new(width, height),
            imgs_textures,
            img_guided_missile,
            img_explosion,
            surfaces,
            map,
            prev_update: 0.0,
            guided_missile,
            explosions: Vec::new(),
            debug_texts: Vec::new(),
        }
    }

    pub fn to_debug_string(&self) -> String {
        format!("{:#?}", self)
    }

    pub fn input(&mut self, cvars: &Cvars, left: f64, right: f64, up: f64, down: f64) {
        // Accel / decel
        let accel =
            up * cvars.g_guided_missile_speed_change - down * cvars.g_guided_missile_speed_change;
        let dir = self.guided_missile.vel.normalized();
        let speed_old = self.guided_missile.vel.magnitude();
        let speed_new = (speed_old + accel).clamped(
            cvars.g_guided_missile_speed_min,
            cvars.g_guided_missile_speed_max,
        );
        self.guided_missile.vel = speed_new * dir;
        self.debug_text(format!("speed {}", speed_new));

        // Turning
        let tr_input: f64 = right * cvars.g_guided_missile_turn_rate_increase
            - left * cvars.g_guided_missile_turn_rate_increase;

        // Without input, turn rate should gradually decrease towards 0
        // but not to turn in the other dir.
        // TODO this doesn't feel like flying a missile - probably needs to carry some sideways momentum
        let tr_old = self.guided_missile.turn_rate;
        let tr = if tr_input == 0.0 {
            if tr_old > 0.0 {
                (tr_old - cvars.g_guided_missile_turn_rate_decrease).max(0.0)
            } else {
                (tr_old + cvars.g_guided_missile_turn_rate_decrease).min(0.0)
            }
        } else {
            (tr_old + tr_input).clamped(
                -cvars.g_guided_missile_turn_rate_max,
                cvars.g_guided_missile_turn_rate_max,
            )
        };

        self.debug_text(format!("tr {}", tr));

        self.guided_missile.vel.rotate_z(tr);
        self.guided_missile.turn_rate = tr;
    }

    pub fn update_pre(&mut self, cvars: &Cvars, t: f64) {
        let dt = t - self.prev_update;
        // TODO this is broken when minimized (collision detection, etc.)

        self.guided_missile.pos += self.guided_missile.vel * dt;
        if self.guided_missile.pos.x <= 0.0 {
            self.impact(cvars);
        }
        if self.guided_missile.pos.y <= 0.0 {
            self.impact(cvars);
        }
        let map_size = self.map.maxs();
        if self.guided_missile.pos.x >= map_size.x {
            self.impact(cvars);
        }
        if self.guided_missile.pos.y >= map_size.y {
            self.impact(cvars);
        }

        let tile_pos = self.map.tile_pos(self.guided_missile.pos);
        let surface = self.map[tile_pos.index].surface;
        let kind = self.surfaces[surface].kind;
        if kind == Kind::Wall {
            self.impact(cvars);
        }

        self.prev_update = t;
    }

    fn impact(&mut self, cvars: &Cvars) {
        self.explosions.push((self.guided_missile.pos, 0));
        self.guided_missile = entities::spawn_guided_missile(cvars, &mut self.rng, &self.map);
    }

    pub fn draw(&mut self, cvars: &Cvars) -> Result<(), JsValue> {
        // Nicer rockets (more like original RW).
        // This also means everything is aligned to pixels.
        // TODO revisit when drawing tanks - maybe make configurable per drawn object
        self.context.set_image_smoothing_enabled(cvars.r_smoothing);

        // Don't put the camera so close to the edge that it would render area outside the map.
        // TODO handle maps smaller than canvas (currently crashes on unreachable)
        let camera_min = self.canvas_size / 2.0;
        let map_size = self.map.maxs();
        let camera_max = map_size - camera_min;
        let camera_pos = self.guided_missile.pos.clamped(camera_min, camera_max);

        let top_left = camera_pos - camera_min;
        let top_left_tp = self.map.tile_pos(top_left);
        let top_left_index = top_left_tp.index;
        let bg_offset = if cvars.r_align_to_pixels_background {
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
        let player_scr_pos = self.guided_missile.pos - top_left;
        // -PI because the img points left
        let angle = self.guided_missile.vel.y.atan2(self.guided_missile.vel.x) - PI;
        self.draw_img(&self.img_guided_missile, player_scr_pos, angle)?;

        // Draw explosions
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
        self.context.fill_rect(screen_pos.x, screen_pos.y, 1.0, 1.0);

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

        /*self.context.save();
        let x = screen_pos.x;
        let y = screen_pos.y;
        let
        self.context.translate(screen_pos.x+img.natural_width()/2, screen_pos.)
        self.context.restore();*/

        Ok(())
    }

    /*function drawImage(ctx, image, x, y, w, h, degrees){
      ctx.save();
      ctx.translate(x+w/2, y+h/2);
      ctx.rotate(degrees*Math.PI/180.0);
      ctx.translate(-x-w/2, -y-h/2);
      ctx.drawImage(image, x, y, w, h);
      ctx.restore();
    }*/

    #[allow(unused)]
    fn debug_text<S: Into<String>>(&mut self, s: S) {
        self.debug_texts.push(s.into());
    }
}

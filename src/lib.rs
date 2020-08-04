// TODO lints

mod cvars;
mod data;
mod entities;
mod logging;

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
    /// Current frame's time in seconds
    frame_time: f64,
    /// Previous frame's time in seconds
    frame_time_prev: f64,
    /// Saved frame times in seconds over some period of time to measure FPS
    frame_times: Vec<f64>,
    input: Input,
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
            frame_time: 0.0,
            frame_time_prev: 0.0,
            frame_times: Vec::new(),
            input: Input::default(),
            guided_missile,
            explosions: Vec::new(),
            debug_texts: Vec::new(),
        }
    }

    pub fn to_debug_string(&self) -> String {
        format!("{:#?}", self)
    }

    /// Update time tracking variables (in seconds)
    pub fn start_frame(&mut self, t: f64) {
        // These two always exist while the vector can get almost empty.
        self.frame_time_prev = self.frame_time;
        self.frame_time = t;

        self.frame_times.push(t);
        while !self.frame_times.is_empty() && self.frame_times[0] + 1.0 < t {
            self.frame_times.remove(0);
        }
    }

    pub fn input(&mut self, cvars: &Cvars, left: f64, right: f64, up: f64, down: f64) {
        self.input = Input {
            left,
            right,
            up,
            down,
        };
    }

    pub fn update_pre(&mut self, cvars: &Cvars) {
        let dt = self.frame_time - self.frame_time_prev;

        // Accel / decel
        let accel = self.input.up * cvars.g_guided_missile_speed_change * dt
            - self.input.down * cvars.g_guided_missile_speed_change * dt;
        let dir = self.guided_missile.vel.normalized();
        let speed_old = self.guided_missile.vel.magnitude();
        let speed_new = (speed_old + accel).clamped(
            cvars.g_guided_missile_speed_min,
            cvars.g_guided_missile_speed_max,
        );
        self.guided_missile.vel = speed_new * dir;
        self.debug_text(format!("GM speed {:.3}", speed_new));

        // Turning
        let tr_input: f64 = self.input.right * cvars.g_guided_missile_turn_rate_increase * dt
            - self.input.left * cvars.g_guided_missile_turn_rate_increase * dt;

        // Without input, turn rate should gradually decrease towards 0
        // but not to turn in the other dir.
        // TODO try a multiplier instead of subtraction
        // TODO this doesn't feel like flying a missile - probably needs to carry some sideways momentum
        let tr_old = self.guided_missile.turn_rate;
        let tr = if tr_input == 0.0 {
            if tr_old > 0.0 {
                (tr_old - cvars.g_guided_missile_turn_rate_decrease * dt).max(0.0)
            } else {
                (tr_old + cvars.g_guided_missile_turn_rate_decrease * dt).min(0.0)
            }
        } else {
            (tr_old + tr_input).clamped(
                -cvars.g_guided_missile_turn_rate_max,
                cvars.g_guided_missile_turn_rate_max,
            )
        };

        self.debug_text(format!("GM turn rate {:.3}", tr));

        self.guided_missile.vel.rotate_z(tr * dt);
        self.guided_missile.turn_rate = tr;

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
                    self.draw_img_top_left(img, Vec2::new(x, y), tile.angle)?;
                }

                c += 1;
                x += TILE_SIZE;
            }
            r += 1;
            y += TILE_SIZE;
        }

        // Draw missile
        let player_scr_pos = self.guided_missile.pos - top_left;
        let angle = self.guided_missile.vel.y.atan2(self.guided_missile.vel.x);
        self.draw_img_center(&self.img_guided_missile, player_scr_pos, angle)?;

        // Draw explosions
        for &(pos, frame) in &self.explosions {
            // TODO frame rate independence
            // It looks like the original animation is made for 30 fps.
            // When stepping through frames of a recording, some images take 3 frames,
            // might be a bug in mplayer though.
            let real_frame = frame / 2;
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
                    self.draw_img_top_left(img, Vec2::new(x, y), tile.angle)?;
                }

                c += 1;
                x += TILE_SIZE;
            }
            r += 1;
            y += TILE_SIZE;
        }

        // Draw FPS
        let fps = if self.frame_times.is_empty() {
            0.0
        } else {
            let diff_time = self.frame_times.last().unwrap() - self.frame_times.first().unwrap();
            let diff_frames = self.frame_times.len() - 1;
            diff_frames as f64 / diff_time
        };
        self.context.fill_text(
            &format!("FPS: {:.1}", fps),
            self.canvas_size.x - 60.0,
            self.canvas_size.y - 15.0,
        )?;

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

    /// Rotate counterclockwise around the image's top left corner.
    fn draw_img_top_left(
        &self,
        img: &HtmlImageElement,
        screen_pos: Vec2f,
        angle: f64,
    ) -> Result<(), JsValue> {
        let half_size = Vec2::new(img.natural_width(), img.natural_height()).as_() / 2.0;
        self.context
            .translate(screen_pos.x + half_size.x, screen_pos.y + half_size.y)?;
        self.context.rotate(angle)?;

        // Now back off to the img's corner and draw it.
        // This can be done either by translating -half_size, then drawing at 0,0
        // or at once by drawing at -half_size which his perhaps marginally more efficient.
        self.context
            .draw_image_with_html_image_element(img, -half_size.x, -half_size.y)?;

        self.context.reset_transform()?;

        Ok(())
    }

    /// Rotate counterclockwise around the image's center.
    fn draw_img_center(
        &self,
        img: &HtmlImageElement,
        screen_pos: Vec2f,
        angle: f64,
    ) -> Result<(), JsValue> {
        let half_size = Vec2::new(img.natural_width(), img.natural_height()).as_() / 2.0;
        self.context.translate(screen_pos.x, screen_pos.y)?;
        self.context.rotate(angle)?;

        // Now back off to the img's corner and draw it.
        // This can be done either by translating -half_size, then drawing at 0,0
        // or at once by drawing at -half_size which his perhaps marginally more efficient.
        self.context
            .draw_image_with_html_image_element(img, -half_size.x, -half_size.y)?;

        self.context.reset_transform()?;

        self.context.fill_rect(screen_pos.x, screen_pos.y, 1.0, 1.0); // TODO remove

        Ok(())
    }

    #[allow(unused)]
    fn debug_text<S: Into<String>>(&mut self, s: S) {
        self.debug_texts.push(s.into());
    }
}

#[derive(Debug, Clone, Default)]
struct Input {
    left: f64,
    right: f64,
    up: f64,
    down: f64,
}

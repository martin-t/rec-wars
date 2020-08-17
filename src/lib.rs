// TODO lints

#[macro_use]
mod debugging;

mod cvars;
mod data;
mod entities;
mod game_state;

use js_sys::Array;

use rand::prelude::*;

use vek::ops::Clamp;
use vek::Vec2;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::{CanvasRenderingContext2d, HtmlImageElement};

use cvars::Cvars;
use data::{Kind, Map, Surface, Vec2f, TILE_SIZE};
use debugging::DEBUG_TEXTS;
use entities::{GuidedMissile, Tank};
use game_state::{GameState, Input, PlayerEntity};

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct World {
    context: CanvasRenderingContext2d,
    canvas_size: Vec2f,
    imgs_textures: Vec<HtmlImageElement>,
    imgs_weapon_icons: Vec<HtmlImageElement>,
    img_gm: HtmlImageElement,
    img_tank: HtmlImageElement,
    img_explosion: HtmlImageElement,
    surfaces: Vec<Surface>,
    map: Map,
    /// Saved frame times in seconds over some period of time to measure FPS
    frame_times: Vec<f64>,
    gs: GameState,
    gs_prev: GameState,
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
        weapon_icons: Array,
        img_gm: HtmlImageElement,
        img_tank: HtmlImageElement,
        img_explosion: HtmlImageElement,
        tex_list_text: &str,
        map_text: &str,
    ) -> Self {
        console_error_panic_hook::set_once();

        let mut rng = SmallRng::seed_from_u64(cvars.d_seed);

        let imgs_textures = textures
            .iter()
            .map(|tile| tile.dyn_into().unwrap())
            .collect();
        let imgs_weapon_icons = weapon_icons
            .iter()
            .map(|js_val| js_val.dyn_into().unwrap())
            .collect();

        let surfaces = data::load_tex_list(tex_list_text);
        let map = data::load_map(map_text, &surfaces);
        let (pos, angle) = entities::random_spawn_pos(&mut rng, &map);

        let gm = GuidedMissile::spawn(cvars, pos, angle);
        let tank = Tank::spawn(pos, angle);
        let pe = PlayerEntity::Tank;

        let gs = GameState {
            rng,
            frame_time: 0.0,
            input: Input::default(),
            gm,
            tank,
            pe,
            explosions: Vec::new(),
        };
        let gs_prev = gs.clone();

        Self {
            context,
            canvas_size: Vec2f::new(width, height),
            imgs_textures,
            imgs_weapon_icons,
            img_gm,
            img_tank,
            img_explosion,
            surfaces,
            map,
            frame_times: Vec::new(),
            gs,
            gs_prev,
        }
    }

    pub fn to_debug_string(&self) -> String {
        format!("{:#?}", self)
    }

    /// Update time tracking variables (in seconds)
    pub fn start_frame(&mut self, t: f64) {
        self.gs_prev = self.gs.clone();
        self.gs.frame_time = t;
        assert!(self.gs.frame_time >= self.gs_prev.frame_time);

        self.frame_times.push(t);
        while !self.frame_times.is_empty() && self.frame_times[0] + 1.0 < t {
            self.frame_times.remove(0);
        }
    }

    pub fn input(&mut self, left: f64, right: f64, up: f64, down: f64, space: bool) {
        self.gs.input = Input {
            left,
            right,
            up,
            down,
            space,
        };
    }

    pub fn update_pre(&mut self, cvars: &Cvars) {
        let dt = self.gs.frame_time - self.gs_prev.frame_time;

        // Tank can shoot while controlling a missile
        if self.gs.input.space && self.gs.tank.charge == 1.0 {
            self.gs.tank.charge = 0.0;
            self.gs.gm = GuidedMissile::spawn(cvars, self.gs.tank.pos, self.gs.tank.angle);
            self.gs.pe = PlayerEntity::GuidedMissile;
        }
        if self.gs.pe == PlayerEntity::Tank {
            self.gs.tank.tick(dt, cvars, &self.gs.input);
        } else {
            self.gs.tank.tick(dt, cvars, &Input::default());
        };

        if self.gs.pe == PlayerEntity::GuidedMissile {
            self.gs.gm.physics(dt, cvars, &self.gs.input);
        } else {
            self.gs.gm.physics(dt, cvars, &Input::default());
        }
        let hit_something = self.gs.gm.collisions(dt, &self.map, &self.surfaces);
        if hit_something {
            self.impact(cvars);
        }
    }

    fn impact(&mut self, cvars: &Cvars) {
        self.gs.explosions.push((self.gs.gm.pos, 0));
        self.gs.pe = PlayerEntity::Tank;
        let (pos, angle) = entities::random_spawn_pos(&mut self.gs.rng, &self.map);
        self.gs.gm = GuidedMissile::spawn(cvars, pos, angle);
    }

    pub fn draw(&self, cvars: &Cvars) -> Result<(), JsValue> {
        // Nicer rockets (more like original RW).
        // This also means everything is aligned to pixels
        // without the need to explicitly round x and y in draw calls to whole numbers.
        // TODO revisit when drawing tanks - maybe make configurable per drawn object
        //      if disabling, try changing quality
        self.context.set_image_smoothing_enabled(cvars.r_smoothing);

        let pe_pos = match self.gs.pe {
            PlayerEntity::Tank => self.gs.tank.pos,
            PlayerEntity::GuidedMissile => self.gs.gm.pos,
        };

        // Don't put the camera so close to the edge that it would render area outside the map.
        // TODO handle maps smaller than canvas (currently crashes on unreachable)
        let camera_min = self.canvas_size / 2.0;
        let map_size = self.map.maxs();
        let camera_max = map_size - camera_min;
        let camera_pos = pe_pos.clamped(camera_min, camera_max);

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
        let gm = &self.gs.gm;
        let player_scr_pos = gm.pos - top_left;
        let gm_angle = gm.vel.y.atan2(gm.vel.x);
        dbgd!(gm_angle.to_degrees());
        self.draw_img_center(&self.img_gm, player_scr_pos, gm_angle)?;
        if cvars.d_debug_draw {
            self.context
                .fill_rect(player_scr_pos.x, player_scr_pos.y, 1.0, 1.0);
        }

        // Draw tank
        // TODO chassis, then cow, then turret
        let tank = &self.gs.tank;
        let tank_scr_pos = tank.pos - top_left;
        self.draw_img_center(&self.img_tank, tank_scr_pos, tank.angle)?;

        // Draw explosions
        for &(pos, frame) in &self.gs.explosions {
            // TODO frame rate independence
            // It looks like the original animation is made for 30 fps.
            // When stepping through frames of a recording, some images take 3 frames,
            // might be a bug in mplayer though.
            // Update: maybe the explosion is supposed to last 31 frames
            // single stepping in blender: 13 sprites, 31 frames:
            //      2,2,3,1,3,3,2,3,2,2,3,2,3
            //      2,2,2,3,1,3,2,2,3,2,2,3,4
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

        // TODO Draw HM indicator using canvas dashed lines

        // Draw HUD
        // TODO canvas shadow methods to draw shadows
        // health: 99x9, goes from greed to red
        self.context.set_fill_style(&"yellow".into());
        self.context.fill_rect(
            cvars.hud_charge_x,
            cvars.hud_charge_y,
            cvars.hud_charge_width * self.gs.tank.charge,
            cvars.hud_charge_height,
        );

        self.context.set_fill_style(&"red".into());

        // Draw FPS
        // TODO this is wrong with d_speed
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
        let mut y = 20.0;
        DEBUG_TEXTS.with(|texts| {
            let mut texts = texts.borrow_mut();
            if cvars.d_debug_text {
                for line in texts.iter() {
                    self.context.fill_text(line, 20.0, y).unwrap();
                    y += 10.0;
                }
            }
            texts.clear();
        });

        Ok(())
    }

    pub fn update_post(&mut self) {
        for explosion in &mut self.gs.explosions {
            explosion.1 += 1;
        }
        self.gs.explosions.retain(|expl| expl.1 < 26);
    }

    /// Rotate clockwise around the image's top left corner.
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
        // or at once by drawing at -half_size which is perhaps marginally more efficient.
        self.context
            .draw_image_with_html_image_element(img, -half_size.x, -half_size.y)?;

        self.context.reset_transform()?;

        Ok(())
    }

    /// Rotate clockwise around the image's center.
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

        Ok(())
    }
}

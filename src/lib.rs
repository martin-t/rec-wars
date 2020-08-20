// TODO lints

#[macro_use]
mod debugging;

mod cvars;
mod entities;
mod game_state;
mod map;

use hecs::World;

use js_sys::Array;

use rand::prelude::*;

use vek::ops::Clamp;
use vek::Vec2;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::{CanvasRenderingContext2d, HtmlImageElement};

use cvars::Cvars;
use debugging::DEBUG_TEXTS;
use entities::{GuidedMissile, Tank};
use game_state::{Explosion, GameState, Input, PlayerEntity};
use map::{Kind, Map, Vec2f, TILE_SIZE};

const WEAP_MG: usize = 0;
const WEAP_RAIL: usize = 1;
const WEAP_CB: usize = 2;
const WEAP_ROCKETS: usize = 3;
const WEAP_HM: usize = 4;
const WEAP_GM: usize = 5;
const WEAP_BFG: usize = 6;
const WEAPS_CNT: usize = 7;

#[derive(Debug)]
struct Pos(Vec2f);

#[derive(Debug)]
struct Vel(Vec2f);

#[wasm_bindgen]
pub struct Game {
    context: CanvasRenderingContext2d,
    canvas_size: Vec2f,
    imgs_textures: Vec<HtmlImageElement>,
    imgs_weapon_icons: Vec<HtmlImageElement>,
    img_gm: HtmlImageElement,
    img_tank: HtmlImageElement,
    img_explosion: HtmlImageElement,
    map: Map,
    /// Saved frame times in seconds over some period of time to measure FPS
    frame_times: Vec<f64>,
    gs: GameState,
    gs_prev: GameState,
    ecs: World,
}

#[wasm_bindgen]
impl Game {
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

        let surfaces = map::load_tex_list(tex_list_text);
        let map = map::load_map(map_text, surfaces);
        let (pos, angle) = entities::random_spawn_pos(&mut rng, &map);

        let gm = GuidedMissile::spawn(cvars, pos, angle);
        let tank = Tank::spawn(pos, angle);
        let pe = PlayerEntity::Tank;

        let gs = GameState {
            rng,
            frame_time: 0.0,
            input: Input::default(),
            cur_weapon: 5,
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
            map,
            frame_times: Vec::new(),
            gs,
            gs_prev,
            ecs: World::new(),
        }
    }

    // TODO Debug for hecs?
    /*pub fn to_debug_string(&self) -> String {
        format!("{:#?}", self)
    }*/

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

    pub fn input(&mut self, input: &Input) {
        self.gs.input = input.clone();
    }

    pub fn update_pre(&mut self, cvars: &Cvars) {
        let dt = self.gs.frame_time - self.gs_prev.frame_time;

        if self.gs.input.change_weapon && !self.gs_prev.input.change_weapon {
            self.gs.cur_weapon = (self.gs.cur_weapon + 1) % WEAPS_CNT;
        }

        // Tank can shoot while controlling a missile
        if self.gs.input.fire && self.gs.tank.charge == 1.0 {
            match self.gs.cur_weapon {
                WEAP_MG => {
                    let pos = Pos(self.gs.tank.pos);
                    let mut vel =
                        Vec2f::new(cvars.g_machine_gun_speed, 0.0).rotated_z(self.gs.tank.angle);
                    if cvars.g_machine_gun_add_vehicle_velocity {
                        vel += self.gs.tank.vel;
                    }
                    self.ecs.spawn((pos, Vel(vel)));
                }
                WEAP_RAIL => {}
                WEAP_CB => {}
                WEAP_ROCKETS => {}
                WEAP_HM => {}
                WEAP_GM => {
                    self.gs.tank.charge = 0.0;
                    self.gs.gm = GuidedMissile::spawn(cvars, self.gs.tank.pos, self.gs.tank.angle);
                    self.gs.pe = PlayerEntity::GuidedMissile;
                }
                WEAP_BFG => {}
                _ => unreachable!("current weapon index out of range"),
            }
        }

        let mut to_remove = Vec::new();
        for (entity, (pos, vel)) in self.ecs.query::<(&mut Pos, &Vel)>().iter() {
            pos.0 += vel.0 * dt;

            if self.map.collision(pos.0) {
                to_remove.push(entity);
            }
        }
        for entity in to_remove {
            self.ecs.despawn(entity).unwrap();
        }

        if self.gs.pe == PlayerEntity::Tank {
            self.gs.tank.tick(dt, cvars, &self.gs.input);
        } else {
            self.gs.tank.tick(dt, cvars, &Input::default());
        };

        let hit_something = if self.gs.pe == PlayerEntity::GuidedMissile {
            self.gs.gm.tick(dt, cvars, &self.gs.input, &self.map)
        } else {
            self.gs.gm.tick(dt, cvars, &Input::default(), &self.map)
        };
        if hit_something {
            let explosion = Explosion::new(self.gs.gm.pos, self.gs.frame_time);
            self.gs.explosions.push(explosion);
            self.gs.pe = PlayerEntity::Tank;
            let (pos, angle) = entities::random_spawn_pos(&mut self.gs.rng, &self.map);
            self.gs.gm = GuidedMissile::spawn(cvars, pos, angle);
        }
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

                if self.map.surface_of(tile).kind != Kind::Wall {
                    let img = &self.imgs_textures[tile.surface_index];
                    self.draw_img_top_left(img, Vec2::new(x, y), tile.angle)?;
                }

                c += 1;
                x += TILE_SIZE;
            }
            r += 1;
            y += TILE_SIZE;
        }

        // Draw MG
        self.context.set_stroke_style(&"yellow".into());
        let mut mg_cnt = 0;
        for (_, (pos, vel)) in self.ecs.query::<(&Pos, &Vel)>().iter() {
            mg_cnt += 1;
            let scr_pos = pos.0 - top_left;
            self.context.begin_path();
            self.context.move_to(scr_pos.x, scr_pos.y);
            // we're drawing from the bullet's position backwards
            let scr_end = scr_pos - vel.0.normalized() * cvars.g_machine_gun_trail_length;
            self.context.line_to(scr_end.x, scr_end.y);
            self.context.stroke();
        }
        dbgd!(mg_cnt);

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
        for explosion in &self.gs.explosions {
            // It looks like the original animation is made for 30 fps.
            // Single stepping a recording of the original RecWars explosion in blender:
            // 13 sprites, 31 frames - examples:
            //      2,2,3,1,3,3,2,3,2,2,3,2,3
            //      2,2,2,3,1,3,2,2,3,2,2,3,4
            // This code produces similar results,
            // though it might display a single sprite for 4 frames slightly more often.
            let progress = (self.gs.frame_time - explosion.start_time) / cvars.g_explosion_duration;
            // 13 sprites in the sheet, 100x100 pixels per sprite
            let frame = (progress * 13.0).floor();
            let offset = frame * 100.0;
            let scr_pos = explosion.pos - top_left;
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

                if self.map.surface_of(tile).kind == Kind::Wall {
                    let img = &self.imgs_textures[tile.surface_index];
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
        self.draw_img_center(
            &self.imgs_weapon_icons[self.gs.cur_weapon],
            Vec2f::new(cvars.hud_weapon_icon_x, cvars.hud_weapon_icon_y),
            0.0,
        )?;

        // let mut scr_pos = Vec2f::new(50.0, 50.0);
        // self.draw_img_top_left(&self.img_tank, scr_pos, 0.0f64.to_radians())?;
        // self.context.stroke_rect(scr_pos.x, scr_pos.y, 50.0, 30.0);
        // scr_pos.y += 100.0;
        // self.draw_img_top_left(&self.img_tank, scr_pos, 45.0f64.to_radians())?;
        // self.context.stroke_rect(scr_pos.x, scr_pos.y, 50.0, 30.0);
        // scr_pos.y += 100.0;
        // self.draw_img_top_left(&self.img_tank, scr_pos, 90.0f64.to_radians())?;
        // self.context.stroke_rect(scr_pos.x, scr_pos.y, 50.0, 30.0);

        // let mut scr_pos = Vec2f::new(150.0, 50.0);
        // self.draw_img_center(&self.img_tank, scr_pos, 0.0f64.to_radians())?;
        // self.context.stroke_rect(scr_pos.x, scr_pos.y, 50.0, 30.0);
        // scr_pos.y += 100.0;
        // self.draw_img_center(&self.img_tank, scr_pos, 45.0f64.to_radians())?;
        // self.context.stroke_rect(scr_pos.x, scr_pos.y, 50.0, 30.0);
        // scr_pos.y += 100.0;
        // self.draw_img_center(&self.img_tank, scr_pos, 90.0f64.to_radians())?;
        // self.context.stroke_rect(scr_pos.x, scr_pos.y, 50.0, 30.0);

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
                    y += cvars.d_debug_text_line_height;
                }
            }
            texts.clear();
        });

        Ok(())
    }

    /// Place the image's *top-left corner* at `screen_pos`,
    /// rotate it clockwise around its center.
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

    /// Place the image's *center* at `screen_pos`,
    /// rotate it clockwise around its center.
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

// TODO lints

use vek::Vec2;

use wasm_bindgen::prelude::*;

use web_sys::{CanvasRenderingContext2d, HtmlImageElement};

type Vec2f = Vec2<f64>;

type Map = Vec<Vec<usize>>;

#[wasm_bindgen]
pub struct World {
    context: CanvasRenderingContext2d,
    canvas_size: Vec2f,
    map: Map,
    pos: Vec2f,
    vel: Vec2f,
    prev_update: f64,
}

#[wasm_bindgen]
impl World {
    #[wasm_bindgen(constructor)]
    pub fn new(context: CanvasRenderingContext2d, width: f64, height: f64) -> Self {
        let mut map = vec![vec![0; 15]; 15];
        map[0][0] = 1;
        map[14][14] = 1;
        Self {
            context,
            canvas_size: Vec2f::new(width, height),
            map: vec![vec![0; 15]; 15],
            pos: Vec2f::new(640.0, 640.0),
            vel: Vec2f::new(0.02, 0.01),
            prev_update: 0.0,
        }
    }

    pub fn input(&mut self, left: f64, right: f64, up: f64, down: f64) {
        self.vel.x -= left * 0.007;
        self.vel.x += right * 0.007;
        self.vel.y -= up * 0.007;
        self.vel.y += down * 0.007;
    }

    pub fn update(&mut self, t: f64) {
        let dt = t - self.prev_update;

        self.pos += self.vel * dt;

        self.prev_update = t;
    }

    pub fn draw(
        &self,
        img_base: &HtmlImageElement,
        img_explosion: &HtmlImageElement,
    ) -> Result<(), JsValue> {
        //TODO let center = self.size / 2.0;

        let pos_in_tile = self.pos % 64.0;
        let pos_tile = (self.pos / 64.0).floor();
        let r = pos_tile.x as usize;
        let c = pos_tile.y as usize;

        // only works properly with positive numbers
        let mut x = -pos_in_tile.x;
        while x < self.canvas_size.x {
            let mut y = -pos_in_tile.y;
            while y < self.canvas_size.y {
                self.context
                    .draw_image_with_html_image_element(img_base, x, y)?;
                y += 64.0;
            }
            x += 64.0;
        }

        Ok(())
    }
}

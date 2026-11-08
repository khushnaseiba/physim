use std::f64::consts::PI;

use web_sys::CanvasRenderingContext2d;

use crate::vec2::Vec2;

pub struct Circle {
    pub pos: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
}

impl Circle {
    pub fn render(&self, ctx: &CanvasRenderingContext2d) {
        ctx.set_fill_style_str("white");
        ctx.begin_path();
        ctx.arc(
            self.pos.x as f64,
            self.pos.y as f64,
            self.radius as f64,
            0.0,
            PI * 2.0,
        )
        .unwrap();
        ctx.fill();
    }

    pub fn tick(&mut self, delta: f32) {
        self.pos += self.velocity * delta;
    }
}

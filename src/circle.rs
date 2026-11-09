use std::f64::consts::PI;

use web_sys::CanvasRenderingContext2d;

use crate::vec2::Vec2;

pub struct Circle {
    pub pos: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
    pub color: String,
}

static mut HUE: u16 = 0;

fn random_color() -> String {
    let hue = unsafe { HUE as f64 };
    unsafe {
        HUE = (HUE + 2) % 360;
    }

    format!("hsl({hue}, 70%, 70%)")
}

impl Circle {
    pub fn new(pos: Vec2, velocity: Vec2, radius: f32) -> Circle {
        Circle {
            pos,
            velocity,
            radius,
            color: random_color(),
        }
    }

    pub fn contains(&self, point: Vec2) -> bool {
        (self.pos - point).magsq() <= self.radius * self.radius
    }

    pub fn render(&self, ctx: &CanvasRenderingContext2d) {
        ctx.set_fill_style_str(&self.color);
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

    pub fn render_outline(&self, ctx: &CanvasRenderingContext2d) {
        ctx.set_stroke_style_str("#222222");
        ctx.set_line_width(3.0);
        ctx.begin_path();
        ctx.arc(
            self.pos.x as f64,
            self.pos.y as f64,
            (self.radius + 2.0) as f64,
            0.0,
            PI * 2.0,
        )
        .unwrap();
        ctx.stroke();
    }

    pub fn tick(&mut self, delta: f32) {
        self.pos += self.velocity * delta;
    }
}

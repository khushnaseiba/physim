use crate::{circle::Circle, prelude::*, vec2::Vec2};

use wasm_bindgen::prelude::*;

mod animation_frame;
mod circle;
mod prelude;
mod vec2;

pub struct GameWindow {
    start: Vec2,
    end: Vec2,
}

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    let document = document();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();

    let ctx: web_sys::CanvasRenderingContext2d = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap();

    let game_width = window().inner_width().unwrap().as_f64().unwrap();
    let game_height = window().inner_height().unwrap().as_f64().unwrap();
    let mut circle = Circle {
        pos: Vec2 {
            x: game_width as f32 / 2.0,
            y: game_height as f32 / 2.0,
        },
        velocity: Vec2 { x: 200.0, y: 400.0 },
        radius: 100.0,
    };

    let mut last_time: f64 = window().performance().unwrap().now();

    let padding: f64 = 100.0;

    animation_frame::run(move |timestamp| {
        let dt = ((timestamp - last_time) / 1000.0) as f32;
        last_time = timestamp;
        circle.simulate(dt);
        let game_width = window().inner_width().unwrap().as_f64().unwrap();
        let game_height = window().inner_height().unwrap().as_f64().unwrap();

        let game_window = GameWindow {
            start: Vec2 {
                x: padding as f32,
                y: padding as f32,
            },
            end: Vec2 {
                x: (game_width - padding) as f32,
                y: (game_height - padding) as f32,
            },
        };

        if (circle.pos.x - circle.radius) < game_window.start.x && circle.velocity.x < 0.0 {
            circle.velocity.x = -circle.velocity.x;
            circle.pos.x = game_window.start.x + circle.radius;
        }

        if (circle.pos.y - circle.radius) < game_window.start.y && circle.velocity.y < 0.0 {
            circle.velocity.y = -circle.velocity.y;
            circle.pos.y = game_window.start.y + circle.radius;
        }

        if (circle.pos.x + circle.radius) > game_window.end.x && circle.velocity.x > 0.0 {
            circle.velocity.x = -circle.velocity.x;
            circle.pos.x = game_window.end.x - circle.radius;
        }

        if (circle.pos.y + circle.radius) > game_window.end.y && circle.velocity.y > 0.0 {
            circle.velocity.y = -circle.velocity.y;
            circle.pos.y = game_window.end.y - circle.radius;
        }

        let canvas_width = canvas.width();
        let canvas_height = canvas.height();

        if (canvas_width as f64) != game_width {
            canvas.set_width(game_width as u32);
        }

        if (canvas_height as f64) != game_height {
            canvas.set_height(game_height as u32);
        }

        ctx.clear_rect(0.0, 0.0, game_width, game_height);
        ctx.set_fill_style_str("#787777");
        ctx.fill_rect(
            padding,
            padding,
            game_width - 2.0 * padding,
            game_height - 2.0 * padding,
        );
        circle.draw(&ctx);
        true
    });
    Ok(())
}

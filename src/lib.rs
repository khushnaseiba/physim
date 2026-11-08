use std::{cell::RefCell, rc::Rc};

use crate::{circle::Circle, game::Game, prelude::*, vec2::Vec2};

use wasm_bindgen::prelude::*;
use web_sys::MouseEvent;

mod animation_frame;
mod circle;
mod game;
mod prelude;
mod vec2;

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    let document = document();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();

    let mut last_time: f64 = window().performance().unwrap().now();

    let game = Rc::new(RefCell::new(Game::new(
        canvas.clone(),
        100.0,
        0.01,
        0.9,
        10.0,
    )));

    {
        let game = Rc::clone(&game);
        animation_frame::run(move |timestamp| {
            let dt = ((timestamp - last_time) / 1000.0) as f32;
            last_time = timestamp;

            for _ in 0..20 {
                game.borrow_mut().tick(dt / 20.0);
            }
            game.borrow().render();

            true
        });
    }

    let callback = Closure::<dyn FnMut(MouseEvent)>::new(move |event: MouseEvent| {
        let x = event.offset_x();
        let y = event.offset_y();

        game.borrow_mut().spawn(Circle {
            pos: Vec2 {
                x: x as f32,
                y: y as f32,
            },
            velocity: Vec2 { x: 0.0, y: 500.0 },
            radius: 40.0,
        });
    });

    canvas.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())?;
    callback.forget();

    Ok(())
}

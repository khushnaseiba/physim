use std::{cell::RefCell, rc::Rc};

use crate::{game::Game, prelude::*};

use wasm_bindgen::prelude::*;
use web_sys::{MouseEvent, WheelEvent};

mod animation_frame;
mod circle;
mod game;
mod prelude;
mod vec2;

fn event_position(event: &MouseEvent) -> (f32, f32) {
    (event.offset_x() as f32, event.offset_y() as f32)
}

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    let document = document();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();

    let mut last_time: f64 = window().performance().unwrap().now();
    let window = window();

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

            game.borrow_mut().update_drag_velocity(dt);
            for _ in 0..20 {
                game.borrow_mut().tick(dt / 20.0);
            }
            game.borrow_mut().update_hover();
            game.borrow().render();

            true
        });
    }

    {
        let game = Rc::clone(&game);
        let callback = Closure::<dyn FnMut(MouseEvent)>::new(move |event: MouseEvent| {
            if event.button() != 0 {
                return;
            }

            let (x, y) = event_position(&event);
            let mut game = game.borrow_mut();
            game.set_mouse_position(x, y);
            game.start_drag();
        });
        canvas.add_event_listener_with_callback("mousedown", callback.as_ref().unchecked_ref())?;
        callback.forget();
    }

    {
        let game = Rc::clone(&game);
        let callback = Closure::<dyn FnMut(MouseEvent)>::new(move |event: MouseEvent| {
            let (x, y) = event_position(&event);
            game.borrow_mut().set_mouse_position(x, y);
        });
        window.add_event_listener_with_callback("mousemove", callback.as_ref().unchecked_ref())?;
        callback.forget();
    }

    {
        let game = Rc::clone(&game);
        let callback = Closure::<dyn FnMut(MouseEvent)>::new(move |event: MouseEvent| {
            if event.button() == 0 {
                game.borrow_mut().end_drag();
            }
        });
        window.add_event_listener_with_callback("mouseup", callback.as_ref().unchecked_ref())?;
        callback.forget();
    }

    {
        let game = Rc::clone(&game);
        let callback = Closure::<dyn FnMut(MouseEvent)>::new(move |event: MouseEvent| {
            event.prevent_default();

            let (x, y) = event_position(&event);
            let mut game = game.borrow_mut();
            game.set_mouse_position(x, y);
            game.delete_selected();
        });
        canvas
            .add_event_listener_with_callback("contextmenu", callback.as_ref().unchecked_ref())?;
        callback.forget();
    }

    {
        let game = Rc::clone(&game);
        let callback = Closure::<dyn FnMut(WheelEvent)>::new(move |event: WheelEvent| {
            event.prevent_default();
            game.borrow_mut()
                .resize_selected(-event.delta_y() as f32 * 0.1);
        });
        canvas.add_event_listener_with_callback("wheel", callback.as_ref().unchecked_ref())?;
        callback.forget();
    }

    Ok(())
}

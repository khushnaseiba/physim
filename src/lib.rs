use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

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

    let window = window();
    let last_time = Rc::new(Cell::new(window.performance().unwrap().now()));
    let paused = Rc::new(Cell::new(!document.has_focus().unwrap_or(true)));

    let game = Rc::new(RefCell::new(Game::new(
        canvas.clone(),
        100.0,
        0.01,
        0.9,
        10.0,
    )));

    {
        let game = Rc::clone(&game);
        let last_time = Rc::clone(&last_time);
        let paused = Rc::clone(&paused);
        animation_frame::run(move |timestamp| {
            if paused.get() {
                last_time.set(timestamp);
                return true;
            }

            let dt = ((timestamp - last_time.get()) / 1000.0) as f32;
            last_time.set(timestamp);

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
        let paused = Rc::clone(&paused);
        let callback = Closure::<dyn FnMut()>::new(move || {
            paused.set(true);
            game.borrow_mut().end_drag();
        });
        window.add_event_listener_with_callback("blur", callback.as_ref().unchecked_ref())?;
        callback.forget();
    }

    {
        let last_time = Rc::clone(&last_time);
        let paused = Rc::clone(&paused);
        let performance = window.performance().unwrap();
        let callback = Closure::<dyn FnMut()>::new(move || {
            last_time.set(performance.now());
            paused.set(false);
        });
        window.add_event_listener_with_callback("focus", callback.as_ref().unchecked_ref())?;
        callback.forget();
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

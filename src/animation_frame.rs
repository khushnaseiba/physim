use crate::prelude::*;

use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::prelude::*;

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("Should register `requestAnimationFrame` OK");
}

pub fn run<F>(mut f: F)
where
    F: FnMut(f64) -> bool + 'static,
{
    let callback = Rc::new(RefCell::new(None));
    let callback_for_frame = Rc::clone(&callback);

    *callback.borrow_mut() = Some(Closure::new(move |timestamp| {
        if f(timestamp) {
            request_animation_frame(callback_for_frame.borrow().as_ref().unwrap());
        } else {
            // Break the callback's self-reference when the animation ends.
            let callback = Rc::clone(&callback_for_frame);
            callback.borrow_mut().take();
        }
    }));

    request_animation_frame(callback.borrow().as_ref().unwrap());
}

pub fn window() -> web_sys::Window {
    web_sys::window().expect("global `window`")
}

pub fn document() -> web_sys::Document {
    window().document().expect("document on `window`")
}

use js_sys::WebAssembly;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
// use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};

// #[wasm_bindgen]
// pub struct Renderer {
//     context: web_sys::WebGlRenderingContext,
//     program: web_sys::WebGlProgram,
//     u_matrix: web_sys::WebGlUniformLocation,
// }

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Use `web_sys`'s global `window` function to get a handle on the global window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let canvas = document.get_element_by_id("mycanvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    // Print some info to the console log
    let width = canvas.width() as f64;
    let height = canvas.height() as f64;
    web_sys::console::log_2(&"Width: %s".into(),&width.into());
    web_sys::console::log_2(&"Height: %s".into(),&height.into());

    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
        // .dyn_into::<WebGlRenderingContext>()?; //

    // Clear the background
    web_sys::console::log_1(&"Clearing the background".into());
    context.set_fill_style(&"white".into());
    context.fill_rect(0.0, 0.0, width, height);

    // Draw some lines
    web_sys::console::log_1(&"Drawing the border".into());
    context.set_stroke_style(&"black".into());
    context.begin_path();
    context.move_to(0.0, 0.0);
    context.line_to(width - 1.0, 0.0);
    context.line_to(width - 1.0, height - 1.0);
    context.line_to(0.0, height - 1.0);
    context.line_to(0.0, 0.0);
    context.stroke();

    Ok(())
}
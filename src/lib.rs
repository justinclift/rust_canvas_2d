use js_sys::WebAssembly;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
// use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};

// #[wasm_bindgen]
// pub struct Renderer {
//     context: web_sys::WebGlRenderingContext,
//     program: web_sys::WebGlProgram,
//     u_matrix: web_sys::WebGlUniformLocation,
// }

struct Point {
    num: i32,
    x:   f64,
    y:   f64,
    z:   f64,
}

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

// Transform the XYZ co-ordinates using the values from the transformation matrix
fn transform(m: Vec<f64>, p: Point) -> Point {
    let top0 = m[0];
    let top1 = m[1];
    let top2 = m[2];
    let top3 = m[3];
    let upper_mid0 = m[4];
    let upper_mid1 = m[5];
    let upper_mid2 = m[6];
    let upper_mid3 = m[7];
    let lower_mid0 = m[8];
    let lower_mid1 = m[9];
    let lower_mid2 = m[10];
    let lower_mid3 = m[11];
    //bot0 := m[12] // The fourth row values are ignored for 3D matrices
    //bot1 := m[13]
    //bot2 := m[14]
    //bot3 := m[15]

    Point {
        num: p.num,
        x: (top0 * p.x) + (top1 * p.y) + (top2 * p.z) + top3,
        y: (upper_mid0 * p.x) + (upper_mid1 * p.y) + (upper_mid2 * p.z) + upper_mid3,
        z: (lower_mid0 * p.x) + (lower_mid1 * p.y) + (lower_mid2 * p.z) + lower_mid3,
    }
}

// Multiplies one matrix by another
fn matrix_mult(opMatrix: Vec<f64>, m: Vec<f64>) -> Vec<f64>{
    let top0 = m[0];
    let top1 = m[1];
    let top2 = m[2];
    let top3 = m[3];
    let upper_mid0 = m[4];
    let upper_mid1 = m[5];
    let upper_mid2 = m[6];
    let upper_mid3 = m[7];
    let lower_mid0 = m[8];
    let lower_mid1 = m[9];
    let lower_mid2 = m[10];
    let lower_mid3 = m[11];
    let bot0 = m[12];
    let bot1 = m[13];
    let bot2 = m[14];
    let bot3 = m[15];

    vec![
    (opMatrix[0] * top0) + (opMatrix[1] * upper_mid0) + (opMatrix[2] * lower_mid0) + (opMatrix[3] * bot0), // 1st col, top
    (opMatrix[0] * top1) + (opMatrix[1] * upper_mid1) + (opMatrix[2] * lower_mid1) + (opMatrix[3] * bot1), // 2nd col, top
    (opMatrix[0] * top2) + (opMatrix[1] * upper_mid2) + (opMatrix[2] * lower_mid2) + (opMatrix[3] * bot2), // 3rd col, top
    (opMatrix[0] * top3) + (opMatrix[1] * upper_mid3) + (opMatrix[2] * lower_mid3) + (opMatrix[3] * bot3), // 4th col, top

    (opMatrix[4] * top0) + (opMatrix[5] * upper_mid0) + (opMatrix[6] * lower_mid0) + (opMatrix[7] * bot0), // 1st col, upper middle
    (opMatrix[4] * top1) + (opMatrix[5] * upper_mid1) + (opMatrix[6] * lower_mid1) + (opMatrix[7] * bot1), // 2nd col, upper middle
    (opMatrix[4] * top2) + (opMatrix[5] * upper_mid2) + (opMatrix[6] * lower_mid2) + (opMatrix[7] * bot2), // 3rd col, upper middle
    (opMatrix[4] * top3) + (opMatrix[5] * upper_mid3) + (opMatrix[6] * lower_mid3) + (opMatrix[7] * bot3), // 4th col, upper middle

    (opMatrix[8] * top0) + (opMatrix[9] * upper_mid0) + (opMatrix[10] * lower_mid0) + (opMatrix[11] * bot0), // 1st col, lower middle
    (opMatrix[8] * top1) + (opMatrix[9] * upper_mid1) + (opMatrix[10] * lower_mid1) + (opMatrix[11] * bot1), // 2nd col, lower middle
    (opMatrix[8] * top2) + (opMatrix[9] * upper_mid2) + (opMatrix[10] * lower_mid2) + (opMatrix[11] * bot2), // 3rd col, lower middle
    (opMatrix[8] * top3) + (opMatrix[9] * upper_mid3) + (opMatrix[10] * lower_mid3) + (opMatrix[11] * bot3), // 4th col, lower middle

    (opMatrix[12] * top0) + (opMatrix[13] * upper_mid0) + (opMatrix[14] * lower_mid0) + (opMatrix[15] * bot0), // 1st col, bottom
    (opMatrix[12] * top1) + (opMatrix[13] * upper_mid1) + (opMatrix[14] * lower_mid1) + (opMatrix[15] * bot1), // 2nd col, bottom
    (opMatrix[12] * top2) + (opMatrix[13] * upper_mid2) + (opMatrix[14] * lower_mid2) + (opMatrix[15] * bot2), // 3rd col, bottom
    (opMatrix[12] * top3) + (opMatrix[13] * upper_mid3) + (opMatrix[14] * lower_mid3) + (opMatrix[15] * bot3)] // 4th col, bottom
}

// Translates (moves) a transformation matrix by the given X, Y and Z values
fn translate(m: Vec<f64>, translate_x: f64, translate_y: f64, translate_z: f64) -> Vec<f64> {
    let translate_matrix = vec![
    1.0, 0.0, 0.0, translate_x,
    0.0, 1.0, 0.0, translate_y,
    0.0, 0.0, 1.0, translate_z,
    0.0, 0.0, 0.0, 1.0];
    matrix_mult(translate_matrix, m)
}
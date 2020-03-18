// use js_sys::WebAssembly;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
// use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};

// #[wasm_bindgen]
// pub struct Renderer {
//     context: web_sys::WebGlRenderingContext,
//     program: web_sys::WebGlProgram,
//     u_matrix: web_sys::WebGlUniformLocation,
// }

enum OperationType {
    NOTHING,
    ROTATE,
    SCALE,
    TRANSLATE,
}

struct Point {
    num: i32,
    x: f64,
    y: f64,
    z: f64,
}

// The 4x4 identity matrix
const IDENTITY_MATRIX: [f64; 16] = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];

// * Globals *

// Initialise the transform matrix to the identity matrix
static mut TRANSFORM_MATRIX: [f64; 16] = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];
static mut OP_TEXT: String = String::new();
static mut QUEUE_OP: &OperationType = &OperationType::NOTHING;

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
    web_sys::console::log_2(&"Width: %s".into(), &width.into());
    web_sys::console::log_2(&"Height: %s".into(), &height.into());

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

// Multiplies one matrix by another
fn matrix_mult(op_matrix: &[f64; 16], m: &[f64; 16]) -> [f64; 16] {
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

    [
        (op_matrix[0] * top0) // 1st col, top
            + (op_matrix[1] * upper_mid0)
            + (op_matrix[2] * lower_mid0)
            + (op_matrix[3] * bot0),
        (op_matrix[0] * top1) // 2nd col, top
            + (op_matrix[1] * upper_mid1)
            + (op_matrix[2] * lower_mid1)
            + (op_matrix[3] * bot1),
        (op_matrix[0] * top2) // 3rd col, top
            + (op_matrix[1] * upper_mid2)
            + (op_matrix[2] * lower_mid2)
            + (op_matrix[3] * bot2),
        (op_matrix[0] * top3) // 4th col, top
            + (op_matrix[1] * upper_mid3)
            + (op_matrix[2] * lower_mid3)
            + (op_matrix[3] * bot3),
        (op_matrix[4] * top0) // 1st col, upper middle
            + (op_matrix[5] * upper_mid0)
            + (op_matrix[6] * lower_mid0)
            + (op_matrix[7] * bot0),
        (op_matrix[4] * top1) // 2nd col, upper middle
            + (op_matrix[5] * upper_mid1)
            + (op_matrix[6] * lower_mid1)
            + (op_matrix[7] * bot1),
        (op_matrix[4] * top2) // 3rd col, upper middle
            + (op_matrix[5] * upper_mid2)
            + (op_matrix[6] * lower_mid2)
            + (op_matrix[7] * bot2),
        (op_matrix[4] * top3) // 4th col, upper middle
            + (op_matrix[5] * upper_mid3)
            + (op_matrix[6] * lower_mid3)
            + (op_matrix[7] * bot3),
        (op_matrix[8] * top0) // 1st col, lower middle
            + (op_matrix[9] * upper_mid0)
            + (op_matrix[10] * lower_mid0)
            + (op_matrix[11] * bot0),
        (op_matrix[8] * top1) // 2nd col, lower middle
            + (op_matrix[9] * upper_mid1)
            + (op_matrix[10] * lower_mid1)
            + (op_matrix[11] * bot1),
        (op_matrix[8] * top2) // 3rd col, lower middle
            + (op_matrix[9] * upper_mid2)
            + (op_matrix[10] * lower_mid2)
            + (op_matrix[11] * bot2),
        (op_matrix[8] * top3) // 4th col, lower middle
            + (op_matrix[9] * upper_mid3)
            + (op_matrix[10] * lower_mid3)
            + (op_matrix[11] * bot3),
        (op_matrix[12] * top0) // 1st col, bottom
            + (op_matrix[13] * upper_mid0)
            + (op_matrix[14] * lower_mid0)
            + (op_matrix[15] * bot0),
        (op_matrix[12] * top1) // 2nd col, bottom
            + (op_matrix[13] * upper_mid1)
            + (op_matrix[14] * lower_mid1)
            + (op_matrix[15] * bot1),
        (op_matrix[12] * top2) // 3rd col, bottom
            + (op_matrix[13] * upper_mid2)
            + (op_matrix[14] * lower_mid2)
            + (op_matrix[15] * bot2),
        (op_matrix[12] * top3) // 4th col, bottom
            + (op_matrix[13] * upper_mid3)
            + (op_matrix[14] * lower_mid3)
            + (op_matrix[15] * bot3),
    ]
}

// Rotates a transformation matrix around the X axis by the given degrees
fn rotate_around_x(m: &[f64; 16], degrees: f64) -> [f64; 16] {
    let rad = degrees.to_radians();
    let rotate_x_matrix = [
        // This is really a 4 x 4 matrix, it's just rustfmt destroys the layout
        //   1.0, 0.0, 0.0, 0.0,
        //   0.0, rad.cos(), -rad.sin(), 0.0,
        //   0.0, rad.sin(), rad.cos(), 0.0,
        //   0.0, 0.0, 0.0, 1.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        rad.cos(),
        -rad.sin(),
        0.0,
        0.0,
        rad.sin(),
        rad.cos(),
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    ];
    matrix_mult(&rotate_x_matrix, m)
}

// Rotates a transformation matrix around the Y axis by the given degrees
fn rotate_around_y(m: &[f64; 16], degrees: f64) -> [f64; 16] {
    let rad = degrees.to_radians();
    let rotate_y_matrix = [
        // This is really a 4 x 4 matrix, it's just rustfmt destroys the layout
        //   rad.cos(), 0.0, rad.sin(), 0.0,
        //   0.0, 1.0, 0.0, 0.0,
        //   -rad.sin(), 0.0, rad.cos(), 0.0,
        //   0.0, 0.0, 0.0, 1.0,
        rad.cos(),
        0.0,
        rad.sin(),
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        -rad.sin(),
        0.0,
        rad.cos(),
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    ];
    matrix_mult(&rotate_y_matrix, m)
}

// Rotates a transformation matrix around the Z axis by the given degrees
fn rotate_around_z(m: &[f64; 16], degrees: f64) -> [f64; 16] {
    let rad = degrees.to_radians();
    let rotate_z_matrix = [
        // This is really a 4 x 4 matrix, it's just rustfmt destroys the layout
        //   rad.cos(), -rad.sin(), 0.0, 0.0,
        //   rad.sin(), rad.cos(), 0.0, 0.0,
        //   0.0, 0.0, 1.0, 0.0,
        //   0.0, 0.0, 0.0, 1.0,
        rad.cos(),
        -rad.sin(),
        0.0,
        0.0,
        rad.sin(),
        rad.cos(),
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    ];
    matrix_mult(&rotate_z_matrix, m)
}

// Scales a transformation matrix by the given X, Y, and Z values
fn scale(m: &[f64; 16], x: f64, y: f64, z: f64) -> [f64; 16] {
    let scale_matrix = [
        // This is really a 4 x 4 matrix, it's just rustfmt destroys the layout
        //   x, 0.0, 0.0, 0.0,
        //   0.0, y, 0.0, 0.0,
        //   0.0, 0.0, z, 0.0,
        //   0.0, 0.0, 0.0, 1.0,
        x, 0.0, 0.0, 0.0, 0.0, y, 0.0, 0.0, 0.0, 0.0, z, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    matrix_mult(&scale_matrix, m)
}

// Set up the details for the transformation operation
unsafe fn set_up_operation(op: &'static OperationType, t: i32, f: i32, x: f64, y: f64, z: f64) {
    let queue_parts = f.clone() as f64; // Number of parts to break each transformation into
    TRANSFORM_MATRIX = IDENTITY_MATRIX.clone(); // Reset the transform matrix
    match op {
        // Rotate the objects in world space
        OperationType::ROTATE => {
            // Divide the desired angle into a small number of parts
            if x != 0.0 {
                TRANSFORM_MATRIX = rotate_around_x(&TRANSFORM_MATRIX, x / queue_parts);
            }
            if y != 0.0 {
                TRANSFORM_MATRIX = rotate_around_y(&TRANSFORM_MATRIX, y / queue_parts);
            }
            if z != 0.0 {
                TRANSFORM_MATRIX = rotate_around_z(&TRANSFORM_MATRIX, z / queue_parts);
            }
            OP_TEXT = format!("Rotation. X: {} Y: {} Z: {}", x, y, z);
        }

        // Scale the objects in world space
        OperationType::SCALE => {
            let mut x_part = 0.0;
            let mut y_part = 0.0;
            let mut z_part = 0.0;
            if x != 1.0 {
                x_part = ((x - 1.0) / queue_parts) + 1.0;
            }
            if y != 1.0 {
                y_part = ((y - 1.0) / queue_parts) + 1.0;
            }
            if z != 1.0 {
                z_part = ((z - 1.0) / queue_parts) + 1.0;
            }
            TRANSFORM_MATRIX = scale(&TRANSFORM_MATRIX, x_part, y_part, z_part);
            OP_TEXT = format!("Scale. X: {} Y: {} Z: {}", x, y, z);
        }

        // Translate (move) the objects in world space
        OperationType::TRANSLATE => {
            TRANSFORM_MATRIX = translate(
                &TRANSFORM_MATRIX,
                x / queue_parts,
                y / queue_parts,
                z / queue_parts,
            );
            OP_TEXT = format!("Translate. X: {} Y: {} Z: {}", x, y, z);
        }

        // Nothing to do
        OperationType::NOTHING => {}
    }
    QUEUE_OP = op;
}

// Transform the XYZ co-ordinates using the values from the transformation matrix
fn transform(m: &[f64; 16], p: Point) -> Point {
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
    //let bot0 = m[12]; // The fourth row values are ignored for 3D matrices
    //let bot1 = m[13];
    //let bot2 = m[14];
    //let bot3 = m[15];

    Point {
        num: p.num,
        x: (top0 * p.x) + (top1 * p.y) + (top2 * p.z) + top3,
        y: (upper_mid0 * p.x) + (upper_mid1 * p.y) + (upper_mid2 * p.z) + upper_mid3,
        z: (lower_mid0 * p.x) + (lower_mid1 * p.y) + (lower_mid2 * p.z) + lower_mid3,
    }
}

// Translates (moves) a transformation matrix by the given X, Y and Z values
fn translate(m: &[f64; 16], translate_x: f64, translate_y: f64, translate_z: f64) -> [f64; 16] {
    let translate_matrix = [
        // This is really a 4 x 4 matrix, it's just rustfmt destroys the layout
        //   1.0, 0.0, 0.0, translate_x,
        //   0.0, 1.0, 0.0, translate_y,
        //   0.0, 0.0, 1.0, translate_z,
        //   0.0, 0.0, 0.0, 1.0,
        1.0,
        0.0,
        0.0,
        translate_x,
        0.0,
        1.0,
        0.0,
        translate_y,
        0.0,
        0.0,
        1.0,
        translate_z,
        0.0,
        0.0,
        0.0,
        1.0,
    ];
    matrix_mult(&translate_matrix, &m)
}

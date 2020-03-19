use std::cell::RefCell;
use std::rc::Rc;

// use js_sys::{WebAssembly};
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

type Edge = Vec<i32>;
type Surface = Vec<i32>;

struct Object {
    c: String,       // Colour of the object
    p: Vec<Point>,   // List of point (vertices) in the object
    e: Vec<Edge>,    // List of points to connect by edges
    s: Vec<Surface>, // List of points to connect in order, to create a surface
    mid: Point, // The mid point of the object.  Used for calculating object draw order in a very simple way
}

impl Object {
    fn new() -> Object {
        Object {
            c: "".to_string(),
            p: Vec::new(),
            e: Vec::new(),
            s: Vec::new(),
            mid: Point {
                num: 0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        }
    }
}

type Matrix = [f64; 16];

// The 4x4 identity matrix
const IDENTITY_MATRIX: Matrix = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];

// * Globals *

// Initialise the transform matrix to the identity matrix
static mut TRANSFORM_MATRIX: Matrix = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];
static mut OP_TEXT: String = String::new();
static mut QUEUE_OP: &OperationType = &OperationType::NOTHING;
static mut POINT_COUNTER: i32 = 0;

// * Helper functions, as the web_sys pieces don't seem capable of being stored in globals *
fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

fn body() -> web_sys::HtmlElement {
    document().body().expect("document should have a body")
}

// Called when the wasm module is instantiated
#[wasm_bindgen]
pub fn wasm_main() -> Result<(), JsValue> {
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

    // Set up the render loop
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        render_frame();

        // Schedule ourself for another requestAnimationFrame callback
        req_anim_frame(window.clone(), f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    Ok(())
}

// Do the rendering here
fn render_frame() {
    // ...
}

// The web_sys bindings (so far) only seem capable of calling request_animation_frame() with a closure :/
fn req_anim_frame(win: web_sys::Window, z: &Closure<dyn FnMut()>) {
    win
        .request_animation_frame(z.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

// Returns an object whose points have been transformed into 3D world space XYZ co-ordinates.  Also assigns a number
// to each point
unsafe fn import_object(ob: Object, x: f64, y: f64, z: f64) -> Object {
    // X and Y translation matrix.  Translates the objects into the world space at the given X and Y co-ordinates
    let translate_matrix = [
        // This is really a 4 x 4 matrix, it's just rustfmt destroys the layout
        // 1.0, 0.0, 0.0, x,
        // 0.0, 1.0, 0.0, y,
        // 0.0, 0.0, 1.0, z,
        // 0.0, 0.0, 0.0, 1.0,
        1.0, 0.0, 0.0, x, 0.0, 1.0, 0.0, y, 0.0, 0.0, 1.0, z, 0.0, 0.0, 0.0, 1.0,
    ];

    // Translate the points
    let mut translated_object = Object::new();
    let mut mid_x = 0.0;
    let mut mid_y = 0.0;
    let mut mid_z = 0.0;
    let mut pt = Point {
        num: 0,
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    for j in ob.p.iter() {
        let pt_x = (translate_matrix[0] * j.x) // 1st col, top
            + (translate_matrix[1] * j.y)
            + (translate_matrix[2] * j.z)
            + (translate_matrix[3] * 1.0);
        let pt_y = (translate_matrix[4] * j.x) // 1st col, upper middle
            + (translate_matrix[5] * j.y)
            + (translate_matrix[6] * j.z)
            + (translate_matrix[7] * 1.0);
        let pt_z = (translate_matrix[8] * j.x) // 1st col, lower middle
            + (translate_matrix[9] * j.y)
            + (translate_matrix[10] * j.z)
            + (translate_matrix[11] * 1.0);
        translated_object.p.push(Point {
            num: POINT_COUNTER,
            x: pt_x,
            y: pt_y,
            z: pt_z,
        });
        mid_x += pt_x;
        mid_y += pt_y;
        mid_z += pt_z;
        POINT_COUNTER += 1;
    }

    // Determine the mid point for the object
    let num_pts = ob.p.len() as f64;
    translated_object.mid.x = mid_x / num_pts;
    translated_object.mid.y = mid_y / num_pts;
    translated_object.mid.z = mid_z / num_pts;

    // Copy the colour, edge, and surface definitions across
    translated_object.c = ob.c;
    for j in ob.e.iter() {
        translated_object.e.push(j.clone());
    }
    for j in ob.s.iter() {
        translated_object.s.push(j.clone());
    }

    translated_object
}

// Multiplies one matrix by another
fn matrix_mult(op_matrix: &Matrix, m: &Matrix) -> Matrix {
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
fn rotate_around_x(m: &Matrix, degrees: f64) -> Matrix {
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
fn rotate_around_y(m: &Matrix, degrees: f64) -> Matrix {
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
fn rotate_around_z(m: &Matrix, degrees: f64) -> Matrix {
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
fn scale(m: &Matrix, x: f64, y: f64, z: f64) -> Matrix {
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
fn transform(m: &Matrix, p: Point) -> Point {
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
fn translate(m: &Matrix, translate_x: f64, translate_y: f64, translate_z: f64) -> Matrix {
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

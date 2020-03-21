use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

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

enum KeyVal {
    KeyNone = 0,
    KeyMoveRight = 1,
    KeyMoveLeft = 2,
    KeyMoveUp = 3,
    KeyMoveDown = 4,
    KeyRotateLeft = 5,
    KeyRotateRight = 6,
    KeyRotateUp = 7,
    KeyRotateDown = 8,
    KeyPageUp = 9,
    KeyPageDown = 10,
    KeyHome = 11,
    KeyEnd = 12,
    KeyMinus = 13,
    KeyPlus = 14,
}

#[derive(Clone)]
struct Point {
    num: i32,
    x: f64,
    y: f64,
    z: f64,
}

type Edge = Vec<i32>;
type Surface = Vec<i32>;

#[derive(Clone)]
struct Object {
    colour: String, // Colour of the object
    points: Vec<Point>, // List of point (vertices) in the object
    edges: Vec<Edge>, // List of points to connect by edges
    surfaces: Vec<Surface>, // List of points to connect in order, to create a surface
    mid_point: Point, // The mid point of the object.  Used for calculating object draw order in a very simple way
}

impl Object {
    fn new() -> Object {
        Object {
            colour: String::new(),
            points: Vec::new(),
            edges: Vec::new(),
            surfaces: Vec::new(),
            mid_point: Point {
                num: 0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        }
    }
}

// struct PaintOrder {
//     mid_z: f64, // Z depth of an object's mid point
//     name: String
// }
//
// impl PaintOrder {
//     fn String(&self) -> String {
//         format!("Name: {}, Mid point: {}", self.name, self.mid_z)
//     }
// }
//
// type PaintOrderSlice = Vec<PaintOrder>;
//
// impl PaintOrderSlice {
//     fn Len(&self) -> usize {
//         self.len()
//     }
//
//     fn Swap(&self, i: i32, j: i32) {
//         (self[i], self[j]) = (self[j], self[i]);
//     }
//
//     fn Less(&self, i: i32, j: i32) -> bool {
//         self[i].mid_z < self[j].mid_z
//     }
// }

// The 4x4 identity matrix
type Matrix = [f64; 16];
const IDENTITY_MATRIX: Matrix = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];
const SOURCE_URL: &str = "https://github.com/justinclift/rust_canvas_2d_test1";
const DEBUG: bool = true;

thread_local! {
    pub static CANVAS: web_sys::HtmlCanvasElement = document()
        .get_element_by_id("mycanvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap();
}

lazy_static! {
    // Initialise some shared state variables (aka globals)
    static ref GRAPH_WIDTH: Arc<Mutex<f64>> = Arc::new(Mutex::new(0.0));
    static ref HEIGHT: Arc<Mutex<f64>> = Arc::new(Mutex::new(0.0));
    static ref HIGHLIGHT_SOURCE: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    static ref OP_TEXT: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    static ref POINT_COUNTER: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
    static ref PREV_KEY: Arc<Mutex<KeyVal>> = Arc::new(Mutex::new(KeyVal::KeyNone));
    static ref QUEUE_OP: Arc<Mutex<OperationType>> = Arc::new(Mutex::new(OperationType::NOTHING));
    static ref STEP_SIZE: Arc<Mutex<f64>> = Arc::new(Mutex::new(0.0));

    // Initialise the transformation matrix from the identity matrix
    static ref TRANSFORM_MATRIX: Arc<Mutex<Matrix>> = Arc::new(Mutex::new(IDENTITY_MATRIX.clone()));
}

// * Helper functions, as the web_sys pieces don't seem capable of being stored in globals *
fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

// fn canvas() -> web_sys::HtmlCanvasElement {
//     document()
//         .get_element_by_id("mycanvas")
//         .unwrap()
//         .dyn_into::<web_sys::HtmlCanvasElement>()
//         .unwrap()
// }

// Main setup
#[wasm_bindgen]
pub fn wasm_main() {
    let width = window().inner_width().unwrap().as_f64().unwrap();
    let height = window().inner_height().unwrap().as_f64().unwrap();

    CANVAS.with(|f| {
        let canvas = &*f;
        canvas.set_attribute("width", &width.to_string());
        canvas.set_attribute("height", &height.to_string());

        // Print some info to the console log
        // web_sys::console::log_2(&"Width attribute: %s".into(), &width_string.into());
        // web_sys::console::log_2(&"Height attribute: %s".into(), &height_string.into());
        // web_sys::console::log_2(&"Width: %s".into(), &width.into());
        // web_sys::console::log_2(&"Height: %s".into(), &height.into());

        // Clear the background
        let ctx = canvas.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();
        ctx.set_fill_style(&"white".into());
        ctx.fill_rect(0.0, 0.0, width, height);

        // Set up the render loop
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            render_frame();
            req_anim_frame(f.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));
        req_anim_frame(g.borrow().as_ref().unwrap());
    });
}

// Simple keyboard handler for catching the arrow, WASD, and numpad keys
// Key value info can be found here: https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/key/Key_Values
#[wasm_bindgen]
pub fn key_press_handler(key_val: i32) {
    if DEBUG {
        web_sys::console::log_2(&"Key is: {}".into(), &key_val.into());
    }

    // // If a key is pressed for a 2nd time in a row, then stop the animated movement
    // if key_val == prevKey && queueOp != OperationType::NOTHING {
    //     set_queue_op(OperationType::NOTHING);
    //     // QUEUE_OP = OperationType::NOTHING;
    //     return
    // }

    // If the plus or minus keys were pressed, increase the step size then cause the current operation to be recalculated
    if key_val == KeyVal::KeyMinus as i32 {
        let mut stp = STEP_SIZE.lock().unwrap();
        *stp -= 5.0;
    } else if key_val == KeyVal::KeyPlus as i32 {
        let mut stp = STEP_SIZE.lock().unwrap();
        *stp += 5.0;
    }

    // Set up translate and rotate operations
    if key_val == KeyVal::KeyMoveLeft as i32 {
        let stp = STEP_SIZE.lock().unwrap();
        set_up_operation(OperationType::TRANSLATE, 50, 12, *stp / 2.0, 0.0, 0.0);
    } else if key_val == KeyVal::KeyMoveRight as i32 {
        let stp = STEP_SIZE.lock().unwrap();
        set_up_operation(OperationType::TRANSLATE, 50, 12, -*stp/2.0, 0.0, 0.0);
    } else if key_val == KeyVal::KeyMoveUp as i32 {
        let stp = STEP_SIZE.lock().unwrap();
        set_up_operation(OperationType::TRANSLATE, 50, 12, 0.0, *stp/2.0, 0.0);
    } else if key_val == KeyVal::KeyMoveDown as i32 {
        let stp = STEP_SIZE.lock().unwrap();
        set_up_operation(OperationType::TRANSLATE, 50, 12, 0.0, -*stp/2.0, 0.0);
    } else if key_val == KeyVal::KeyRotateLeft as i32 {
        let stp = STEP_SIZE.lock().unwrap();
        set_up_operation(OperationType::ROTATE, 50, 12, 0.0, -*stp, 0.0);
    } else if key_val == KeyVal::KeyRotateRight as i32 {
        let stp = STEP_SIZE.lock().unwrap();
        set_up_operation(OperationType::ROTATE, 50, 12, 0.0, *stp, 0.0);
    } else if key_val == KeyVal::KeyRotateUp as i32 {
        let stp = STEP_SIZE.lock().unwrap();
        set_up_operation(OperationType::ROTATE, 50, 12, -*stp, 0.0, 0.0);
    } else if key_val == KeyVal::KeyRotateDown as i32 {
        let stp = STEP_SIZE.lock().unwrap();
        set_up_operation(OperationType::ROTATE, 50, 12, *stp, 0.0, 0.0);
    } else if key_val == KeyVal::KeyPageUp as i32 {
        let stp = STEP_SIZE.lock().unwrap();
        set_up_operation(OperationType::ROTATE, 50, 12, -*stp, *stp, 0.0);
    } else if key_val == KeyVal::KeyPageDown as i32 {
        let stp = STEP_SIZE.lock().unwrap();
        set_up_operation(OperationType::ROTATE, 50, 12, *stp, *stp, 0.0);
    } else if key_val == KeyVal::KeyHome as i32 {
        let stp = STEP_SIZE.lock().unwrap();
        set_up_operation(OperationType::ROTATE, 50, 12, -*stp, -*stp, 0.0);
    } else if key_val == KeyVal::KeyEnd as i32 {
        let stp = STEP_SIZE.lock().unwrap();
        set_up_operation(OperationType::ROTATE, 50, 12, *stp, -*stp, 0.0);
    }
    // prev_key = key_val;
}

// Simple mouse handler watching for people moving the mouse over the source code link
#[wasm_bindgen]
pub fn move_handler(cx: i32, cy: i32) {
    let client_x = cx as f64;
    let client_y = cy as f64;
    if DEBUG {
        web_sys::console::log_3(&"client_x: %s, client_y: %s".into(), &client_x.into(), &client_y.into());
    }

    // If the mouse is over the source code link, let the frame renderer know to draw the url in bold
    let height = HEIGHT.lock().unwrap();
    let graph_width = GRAPH_WIDTH.lock().unwrap();
    if (client_x > *graph_width) && (client_y > *height - 40.0) {
        let mut high_light_source = HIGHLIGHT_SOURCE.lock().unwrap();
        *high_light_source = true;
    } else {
        let mut high_light_source = HIGHLIGHT_SOURCE.lock().unwrap();
        *high_light_source = false;
    }
}

// Do the rendering here
fn render_frame() {
    CANVAS.with(|f| {
        let canvas = &*f;
        let mut width = canvas.width() as f64;
        let mut height = canvas.height() as f64;

        // Handle window resizing
        let current_body_width = window().inner_width().unwrap().as_f64().unwrap();
        let current_body_height = window().inner_height().unwrap().as_f64().unwrap();
        if current_body_width != width || current_body_height != height {
            width = current_body_width;
            height = current_body_height;
            canvas.set_attribute("width", &width.to_string());
            canvas.set_attribute("height", &height.to_string());
        }

        // Get the 2D context for the canvas
        let ctx = canvas.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();

        // Setup useful variables
        let border = 2.0;
        let gap = 3.0;
        let left = border + gap;
        let top = border + gap;
        let mut graph_width = GRAPH_WIDTH.lock().unwrap();
        *graph_width = width * 0.75;
        let graph_height = height - 1.0;
        let center_x= *graph_width / 2.0;
        let center_y = graph_height / 2.0;

        // Clear the background
        ctx.set_fill_style(&"white".into());
        ctx.fill_rect(0.0, 0.0, width, height);

        // Save the current graphics state - no clip region currently defined - as the default
        ctx.save();

        // Set the clip region so drawing only occurs in the display area
        ctx.begin_path();
        ctx.move_to(0.0, 0.0);
        ctx.line_to(*graph_width, 0.0);
        ctx.line_to(*graph_width, height);
        ctx.line_to(0.0, height);
        ctx.clip();

        // * Draw grid lines *

        let step = width.min(height) / 30.0;
        ctx.set_stroke_style(&"rgb(220, 220, 220)".into());

        // We use while loops here, because Rust doesn't seem able to iterate using an f64 step. eg .step_by(step)
        // At least not yet: "the trait `std::iter::Step` is not implemented for `f64`"

        // Vertical dashed lines
        let mut i = left;
        while i < *graph_width-step {
            ctx.begin_path();
            ctx.move_to(i+step, top);
            ctx.line_to(i+step, graph_height);
            ctx.stroke();
            i += step;
        }

        // Horizontal dashed lines
        let mut i = top;
        while i < graph_height-step {
            ctx.begin_path();
            ctx.move_to(left, i+step);
            ctx.line_to(*graph_width-border, i+step);
            ctx.stroke();
            i += step;
        }

        // The point objects
        let object1 = Object {
            colour: "lightblue".into(),
            points: vec![
                Point {num: 0, x: 0.0, y: 1.75, z: 1.0}, // Point 0 for this object
                Point {num: 1, x: 1.5, y: -1.75, z: 1.0}, // Point 1 for this object
                Point {num: 2, x: -1.5, y: -1.75, z: 1.0}, // etc
                Point {num: 3, x: 0.0, y: 0.0, z: 1.75},
            ],
            edges: vec![
                vec![0, 1], // Connect point 0 to point 1 to define an edge
                vec![0, 2], // Connect point 0 to point 2 to define an edge
                vec![1, 2], // Connect point 1 to point 2 to define an edge
                vec![0, 3], // etc
                vec![1, 3],
                vec![2, 3],
            ],
            surfaces: vec![
                vec![0, 1, 3], // Connect edge 0, 1, and 3 to define a surface
                vec![0, 2, 3], // etc
                vec![0, 1, 2],
                vec![1, 2, 3],
            ],
            mid_point: Point {num: 0, x: 0.0, y: 0.0, z: 0.0},
        };

        let high_light_source = false;

        // The empty world space
        let point_counter = 1;
        let mut world_space = HashMap::new();
        let (z, _point_counter) = import_object(&object1, point_counter, 5.0, 3.0, 0.0);
        world_space.insert("ob1", &z);

        // // Sort the objects by mid point Z depth order
        // let order = paintOrderSlice;
        // for i, j := range worldSpace {
        //     order = append(order, paintOrder{name: i, midZ: j.Mid.Z})
        // }
        // sort.Sort(paintOrderSlice(order))

        // Draw the objects, in Z depth order
        let mut point_x;
        let mut point_y;
        let num_worlds = world_space.len();
        for _i in 0..num_worlds {
            let obj = &object1;
            // let obj = match world_space.get(&"obj1") {
            //     Some(&thing) => thing,
            //     _ => (),
            // };
            // let obj = world_space[i];
            //     let o = world_space[order[i].name];

            // Draw the surfaces
            ctx.set_fill_style(&format!("{}", obj.colour).into());
            for surf in obj.surfaces.iter() {
                for (m, n) in surf.iter().enumerate() {
                    point_x = obj.points[*n as usize].x;
                    point_y = obj.points[*n as usize].y;
                    if m == 0 {
                        ctx.begin_path();
                        ctx.move_to(center_x + (point_x * step), center_y + ((point_y * step) * -1.0));
                    } else {
                        ctx.line_to(center_x + (point_x * step), center_y + ((point_y * step) * -1.0));
                    }
                }
                ctx.close_path();
                ctx.fill();
            }

            // Draw the edges
            ctx.set_stroke_style(&"black".into());
            ctx.set_fill_style(&"black".into());
            ctx.set_line_width(1.0);
            let mut point1_x;
            let mut point1_y;
            let mut point2_x;
            let mut point2_y;
            for edge in obj.edges.iter() {
                point1_x = obj.points[edge[0 as usize] as usize].x;
                point1_y = obj.points[edge[0 as usize] as usize].y;
                point2_x = obj.points[edge[1 as usize] as usize].x;
                point2_y = obj.points[edge[1 as usize] as usize].y;
                ctx.begin_path();
                ctx.move_to(center_x+(point1_x*step), center_y+((point1_y*step)*-1.0));
                ctx.line_to(center_x+(point2_x*step), center_y+((point2_y*step)*-1.0));
                ctx.stroke();
            }

            // Draw the points on the graph
            let mut px;
            let mut py;
            for point in obj.points.iter() {
                px = center_x + (point.x * step);
                py = center_y + ((point.y * step) * -1.0);
                ctx.begin_path();
                ctx.arc(px, py, 1.0, 0.0, 2.0 * std::f64::consts::PI);
                ctx.fill();
            }
        }

        // Set the clip region so drawing only occurs in the display area
        ctx.restore();
        ctx.save();
        ctx.begin_path();
        ctx.move_to(*graph_width, 0.0);
        ctx.line_to(width, 0.0);
        ctx.line_to(width, height);
        ctx.line_to(*graph_width, height);
        ctx.clip();

        // Draw the text describing the current operation
        let mut text_y = top + 20.0;
        ctx.set_fill_style(&"black".into());
        ctx.set_font(&"bold 14px serif");
        ctx.fill_text("Operation:", *graph_width + 20.0, text_y);
        text_y += 20.0;
        ctx.set_font(&"14px sans-serif");
        {
            let op_text = OP_TEXT.lock().unwrap();
            ctx.fill_text(&*op_text, *graph_width + 20.0, text_y);
        }
        text_y += 30.0;

        // Add the help text about control keys and mouse zoom
        ctx.set_fill_style(&"blue".into());
        ctx.set_font(&"14px sans-serif");
        ctx.fill_text("Use wasd to move, numpad keys", *graph_width + 20.0, text_y);
        text_y += 20.0;
        ctx.fill_text("to rotate, mouse wheel to zoom.", *graph_width + 20.0, text_y);
        text_y += 30.0;
        ctx.fill_text("+ and - keys to change speed.", *graph_width + 20.0, text_y);
        text_y += 30.0;
        ctx.fill_text("Press a key a 2nd time to", *graph_width + 20.0, text_y);
        text_y += 20.0;
        ctx.fill_text("stop the current change.", *graph_width + 20.0, text_y);
        text_y += 40.0;

        // Clear the source code link area
        ctx.set_fill_style(&"white".into());
        ctx.fill_rect(*graph_width + 1.0, graph_height - 55.0, width, height);

        // Add the URL to the source code
        ctx.set_fill_style(&"black".into());
        ctx.set_font("bold 14px serif".into());
        ctx.fill_text("Source code:", *graph_width + 20.0, graph_height - 35.0);
        ctx.set_fill_style(&"blue".into());
        if high_light_source == true {
            ctx.set_font("bold 12px sans-serif".into());
        } else {
            ctx.set_font("12px sans-serif".into());
        }
        ctx.fill_text(SOURCE_URL, *graph_width + 20.0, graph_height - 15.0);

        // Draw a border around the graph area
        ctx.set_line_width(2.0);
        ctx.set_stroke_style(&"white".into());
        ctx.begin_path();
        ctx.move_to(0.0, 0.0);
        ctx.line_to(width, 0.0);
        ctx.line_to(width, height);
        ctx.line_to(0.0, height);
        ctx.close_path();
        ctx.stroke();
        ctx.set_line_width(2.0);
        ctx.set_stroke_style(&"black".into());
        ctx.begin_path();
        ctx.move_to(border, border);
        ctx.line_to(*graph_width, border);
        ctx.line_to(*graph_width, graph_height);
        ctx.line_to(border, graph_height);
        ctx.close_path();
        ctx.stroke();

        // Restore the default graphics state (eg no clip region)
        ctx.restore();
    });
}

// The web_sys bindings (so far) only seem capable of calling request_animation_frame() with a closure :/
fn req_anim_frame(z: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(z.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

// Returns an object whose points have been transformed into 3D world space XYZ co-ordinates.  Also assigns a number
// to each point
fn import_object(ob: &Object, mut point_counter: i32, x: f64, y: f64, z: f64) -> (Object, i32) {
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
    for j in ob.points.iter() {
        pt.x = (translate_matrix[0] * j.x) // 1st col, top
            + (translate_matrix[1] * j.y)
            + (translate_matrix[2] * j.z)
            + (translate_matrix[3] * 1.0);
        pt.y = (translate_matrix[4] * j.x) // 1st col, upper middle
            + (translate_matrix[5] * j.y)
            + (translate_matrix[6] * j.z)
            + (translate_matrix[7] * 1.0);
        pt.z = (translate_matrix[8] * j.x) // 1st col, lower middle
            + (translate_matrix[9] * j.y)
            + (translate_matrix[10] * j.z)
            + (translate_matrix[11] * 1.0);
        translated_object.points.push(Point {
            num: point_counter,
            x: pt.x,
            y: pt.y,
            z: pt.z,
        });
        mid_x += pt.x;
        mid_y += pt.y;
        mid_z += pt.z;
        point_counter += 1;
    }

    // Determine the mid point for the object
    let num_pts = ob.points.len() as f64;
    translated_object.mid_point.x = mid_x / num_pts;
    translated_object.mid_point.y = mid_y / num_pts;
    translated_object.mid_point.z = mid_z / num_pts;

    // Copy the colour, edge, and surface definitions across
    translated_object.colour = ob.colour.clone();
    for j in ob.edges.iter() {
        translated_object.edges.push(j.clone());
    }
    for j in ob.surfaces.iter() {
        translated_object.surfaces.push(j.clone());
    }

    (translated_object, point_counter)
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
fn set_up_operation(op: OperationType, _t: i32, f: i32, x: f64, y: f64, z: f64) {
    let queue_parts = f.clone() as f64; // Number of parts to break each transformation into
    let mut transformation_matrix = TRANSFORM_MATRIX.lock().unwrap(); // Unlock the mutex
    *transformation_matrix = IDENTITY_MATRIX.clone(); // Reset the transform matrix
    match op {
        // Rotate the objects in world space
        OperationType::ROTATE => {
            // Divide the desired angle into a small number of parts
            if x != 0.0 {
                *transformation_matrix = rotate_around_x(&*transformation_matrix, x / queue_parts);
            }
            if y != 0.0 {
                *transformation_matrix = rotate_around_y(&*transformation_matrix, y / queue_parts);
            }
            if z != 0.0 {
                *transformation_matrix = rotate_around_z(&*transformation_matrix, z / queue_parts);
            }
            let mut op_text = OP_TEXT.lock().unwrap(); // Unlocks the mutex
            *op_text = format!("Rotation. X: {} Y: {} Z: {}", x, y, z); // Sets the new value
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
            *transformation_matrix = scale(&*transformation_matrix, x_part, y_part, z_part);
            let mut op_text = OP_TEXT.lock().unwrap(); // Unlocks the mutex
            *op_text = format!("Scale. X: {} Y: {} Z: {}", x, y, z); // Sets the new value
        }

        // Translate (move) the objects in world space
        OperationType::TRANSLATE => {
            *transformation_matrix = translate(
                &*transformation_matrix,
                x / queue_parts,
                y / queue_parts,
                z / queue_parts,
            );
            let mut op_text = OP_TEXT.lock().unwrap(); // Unlocks the mutex
            *op_text = format!("Translate. X: {} Y: {} Z: {}", x, y, z); // Sets the new value
        }

        // Nothing to do
        OperationType::NOTHING => {}
    }

    let mut queue_op = QUEUE_OP.lock().unwrap(); // Unlocks the mutex
    *queue_op = op; // Sets the new value
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

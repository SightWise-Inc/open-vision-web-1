#[allow(unused_imports)]

mod pixel;
mod transforms;

use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use web_sys::{CanvasRenderingContext2d, ImageData};

use std::rc::Rc;
use std::cell::RefCell;

use handpose_rs::{Handpose, visualize, HandResult};
use image::{DynamicImage, RgbaImage, ImageBuffer};
use serde::{Serialize, Deserialize};
use js_sys::{Function, JsString, Uint8ClampedArray};

use tiny_skia::{PremultipliedColorU8};

use once_cell::sync::Lazy;
use std::sync::Mutex;

// static mut HANDPOSE: Handpose = Handpose::new().unwrap();

static HANDPOSE: Lazy<Mutex<Handpose>> = Lazy::new(|| {
    Mutex::new(Handpose::new().unwrap())
});


#[wasm_bindgen]
pub enum Transformation {
    Pixelate,
    Greyscale,
    Unknown,
}

#[wasm_bindgen]
pub fn transform(
    source: &CanvasRenderingContext2d,
    output: &CanvasRenderingContext2d,
    width: u32,
    height: u32,
    square_size: u32,
    transform: Transformation,
) -> Result<(), JsValue> {
    let transform_fn = match transform {
        Transformation::Pixelate => transforms::color_average,
        Transformation::Greyscale => transforms::average,
        _ => transforms::identity,
    };

    let source_data = source
        .get_image_data(0.0, 0.0, width.into(), height.into())?
        .data();


    let mut output_data =
        transforms::apply_transform(&source_data, width, height, square_size, transform_fn);

    let output_image_data =
        ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut output_data), width, height)?;

    output.put_image_data(&output_image_data, 0.0, 0.0)
}

#[wasm_bindgen]
pub fn transform_handpose(
    // hp_closure: JsValue,
    // hp_closure: &Function,
    source: &CanvasRenderingContext2d,
    output: &CanvasRenderingContext2d,
    width: u32,
    height: u32,
    square_size: u32,
    transform: Transformation,


) -> Result<(), JsValue> {
    let transform_fn = match transform {
        Transformation::Pixelate => transforms::color_average,
        Transformation::Greyscale => transforms::average,
        _ => transforms::identity,
    };

    let source_data = source
        .get_image_data(0.0, 0.0, width.into(), height.into())?
        .data();


    // let this = JsValue::null();
    // let hp = hp_closure.call0(&this).unwrap();
    // let mut hp;
    // unsafe { hp = HANDPOSE; }
    let hp = HANDPOSE.lock().unwrap();

    // let image_vec = image_data_to_vec(&source_data);
    let image_vec = unclamp_vec_u8(source_data);
    let image = vec_to_dynamic_image(image_vec, width, height);

    let results = hp.process(image.clone()).unwrap();

    let ldmk_coords = results[0].landmark.coords.clone();
    let pixmap = visualize::visualize2(image.clone(), ldmk_coords.clone()); // ALT1
    let pixels = pixmap.pixels();

    // let mut output_data =
    //     transforms::apply_handpose(&source_data, width, height, square_size, transform_fn);
    let mut output_data = premultiplied_colors_to_u8_array(&pixels);

    let output_image_data =
        ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut output_data), width, height)?;

    output.put_image_data(&output_image_data, 0.0, 0.0)
}

// =========
// closures
// =========

// #[wasm_bindgen]
// pub fn create_hp_closure(value: u32) -> Result<JsValue, JsValue> {
//     let shared_value = Rc::new(RefCell::new(value));
//     // Initialize Handpose here, assuming its constructor does not require parameters
//     // or only requires parameters that are available in Rust.
//     let handpose = Handpose::new(); // Adjust according to the actual constructor
//     let shared_handpose = Rc::new(RefCell::new(handpose));
//     let closure = Closure::wrap(Box::new(move || {
//         let value = shared_value.borrow();
//         let hp = shared_handpose.borrow();
//         // Use hp as needed
//         web_sys::console::log_1(&format!("Value in closure: {}", *value).into());
//     }) as Box<dyn FnMut()>);
//     Ok(closure.into_js_value())
// }

// #[wasm_bindgen]
// pub fn create_model_closure() -> Closure<dyn FnMut(&[f32], &Closure<dyn FnMut(JsValue)>)> {
//     let handpose = Handpose::new().unwrap();
//     let model = Rc::new(handpose);
//     let closure = Closure::wrap(Box::new(move |input, callback| {
//         let output = model.process(input);
//         callback.call(JsValue::from_serde(&output).unwrap());
//     }) as Box<dyn FnMut(&[f32], &Closure<dyn FnMut(JsValue)>)>);
//     closure
// }

// =========
// helpers
// =========

// fn vec_to_image(data: Vec<u8>, width: u32, height: u32) -> RgbaImage {
//     let mut imgbuf = ImageBuffer::new(width, height);
//     for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
//         let base = (y * width + x) as usize * 4;
//         *pixel = image::Rgba([
//             data[base],
//             data[base + 1],
//             data[base + 2],
//             data[base + 3],
//         ]);
//     }
//     imgbuf
// }

fn vec_to_dynamic_image(data: Vec<u8>, width: u32, height: u32) -> DynamicImage {
    let mut imgbuf = ImageBuffer::new(width, height);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let base = (y * width + x) as usize * 4;
        *pixel = image::Rgba([
            data[base],
            data[base + 1],
            data[base + 2],
            data[base + 3],
        ]);
    }
    DynamicImage::ImageRgba8(imgbuf)
}

fn image_data_to_vec(image_data: &ImageData) -> Vec<u8> {
    image_data.data().to_vec().into_iter().map(|x| x.into()).collect()
}

#[wasm_bindgen]
pub fn unclamp_vec_u8(clamped_vec: Clamped<Vec<u8>>) -> Vec<u8> {
    let mut normal_vec = Vec::with_capacity(clamped_vec.len());
    for byte in clamped_vec.iter() {
        normal_vec.push(*byte);
    }
    normal_vec
}


fn premultiplied_colors_to_u8_array(colors: &[PremultipliedColorU8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(colors.len() * 4); // Each color is 4 bytes

    for color in colors {
        let demultiplied = color.demultiply();
        result.push(demultiplied.red());
        result.push(demultiplied.green());
        result.push(demultiplied.blue());
        result.push(demultiplied.alpha());
    }

    result
}

// /// Closure Experiment
// #[wasm_bindgen]
// pub fn create_closure(value: u32) -> JsValue {
//     let shared_value = Rc::new(RefCell::new(value));
//     let closure = Closure::wrap(Box::new(move || {
//         let value = shared_value.borrow();
//         web_sys::console::log_1(&JsValue::from_str(&format!("Value in closure: {}", *value)));
//     }) as Box<dyn FnMut()>);
//     // Convert the Closure into a JsValue before returning
//     closure.into_js_value()
// }

// #[wasm_bindgen]
// pub fn use_value(closure: &Closure<dyn FnMut()>) {
//     closure.call();
// }
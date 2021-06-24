use image::{ImageBuffer, Rgb};
use num_complex::Complex;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Instant;

pub type OutBuffer = ImageBuffer<Rgb<u8>, Vec<u8>>;

pub fn simple_julia() -> OutBuffer {
    // Oh lol:
    // https://crates.io/crates/image
    let imgx = 800;
    let imgy = 800;

    let scalex = 3.0 / imgx as f32;
    let scaley = 3.0 / imgy as f32;

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    // Iterate over the coordinates and pixels of the image
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let r = (0.3 * x as f32) as u8;
        let b = (0.3 * y as f32) as u8;
        *pixel = image::Rgb([r, 0, b]);

        let cx = y as f32 * scalex - 1.5;
        let cy = x as f32 * scaley - 1.5;

        let c = Complex::new(-0.4f32, 0.6f32);
        let mut z = Complex::new(cx, cy);

        let mut i = 0;
        while i < 255 && z.norm() <= 2.0 {
            z = z * z + c;
            i += 1;
        }

        let image::Rgb(data) = *pixel;
        *pixel = image::Rgb([data[0], i as u8, data[2]]);
    }

    imgbuf
}

pub fn mandelbrot() -> OutBuffer {
    let imgx = 800;
    let imgy = 800;

    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    const LIMIT: u32 = 200;

    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let y0 = (y as f64 / imgy as f64) * 4.0 - 2.0;
        let x0 = (x as f64 / imgx as f64) * 3.0 - 2.0;

        let mut x = 0.0;
        let mut y = 0.0;
        let mut iteration = 0;

        let mut x2 = 0.0;
        let mut y2 = 0.0;
        let mut sum = 0.0;

        while sum < 4.0 && iteration < LIMIT {
            y = (x + x) * y + y0;

            x = x2 - y2 + x0;

            x2 = x * x;

            y2 = y * y;

            sum = x2 + y2;

            iteration += 1;
        }

        *pixel = color_rainbow(iteration, LIMIT);
    }

    imgbuf
}

fn color_gray(iteration: u32, limit: u32) -> image::Rgb<u8> {
    let lum = (iteration as f32 / limit as f32 * 255.0) as u8;
    image::Rgb([lum, lum, lum])
}

fn color_rainbow(iteration: u32, limit: u32) -> image::Rgb<u8> {
    // TODO: variable names are nonsense, refactor

    let mut pixel = image::Rgb([0, 0, 0]);

    if iteration < limit {
        let mut h = iteration as f64 % 360.0;
        h /= 60.0;
        let i = h as usize;
        let mut f = h - i as f64; // factorial part of h
        let mut q = 1.0 - f;

        f *= 255.0;
        q *= 255.0;

        let r_arr = [255, q as u8, 0, 0, f as u8, 255];
        let g_arr = [f as u8, 255, 255, q as u8, 0, 0];
        let b_arr = [0, 0, f as u8, 255, 255, q as u8];

        pixel = image::Rgb([r_arr[i], g_arr[i], b_arr[i]])
    }

    pixel
}

pub fn run_on_thread() -> Receiver<OutBuffer> {
    let (sender, receiver) = channel();

    thread::spawn(move || loop {
        let start = Instant::now();
        let image = mandelbrot();
        
        println!("Render took {}", start.elapsed().as_millis());

        sender.send(image).unwrap();
    });

    receiver
}

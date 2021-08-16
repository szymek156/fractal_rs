use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};

use image::Rgb;

use crate::fractal_builder::Context;

/// Trait defining underlying floating type
// Send and Sync to safely pass type over threads
// TODO: maybe there is already numeric trait, which fulfills all those?
// Mul<Output=Self> means, Type has to implement Mul, and result of this operation also needs to be Floating
// Thats not always the case - see Rug implementation
pub trait Floating = From<f64>
    + Copy
    + 'static
    + MulAssign
    + Mul<Output = Self>
    // + Div<Output = Self>
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + PartialOrd
    + Send
    + Sync
    + Debug;

/// PoI - point of interest on a complex plane
#[derive(Debug, Default)]
pub struct PoI<Floating> {
    pub origin_x: Floating,
    pub origin_y: Floating,
    pub pinhole_size: Floating,
    pub limit: u32,
}

/// Interface required for fractal to be implemented for drawing purposes
/// Struct which implements this trait, are constrained to be Sync + Send,
/// That impacts also F type.
pub trait FractalFunction<F: Floating>: Send + Sync {
    // &self to have safe object
    fn draw(&self, context: &Context<F>, id: u32, height: u32, pixels: &mut [Rgb<u8>]);
}

// Unused type parameters cause some internal compiler problems
// that I do not understand, and they were made illegal long time
// ago. _marker is zero sized type, that pretends usage of Floating,
// making compiler happy.
pub struct Mandelbrot<F>(pub PhantomData<F>);

impl<F: Floating> FractalFunction<F> for Mandelbrot<F> {
    fn draw(&self, context: &Context<F>, id: u32, height: u32, pixels: &mut [Rgb<u8>]) {
        let imgx = context.img_width as f64;
        let imgy = context.img_height as f64;
        let pinhole_center = context.poi.pinhole_size * F::from(0.5);

        let center_x = context.poi.origin_x - pinhole_center;
        let center_y = context.poi.origin_y - pinhole_center;

        let four: F = F::from(4.0);

        //TODO: range span?? calc min and max
        for pixel_y in 0..height {
            let y_offset = (pixel_y + id * height) as f64;
            let y0 = F::from(y_offset / imgy) * context.poi.pinhole_size + center_y;

            // TODO: this repeats every row, store value in an array?
            for pixel_x in 0..context.img_width {
                let x0 = F::from(pixel_x as f64 / imgx) * context.poi.pinhole_size + center_x;

                let mut x = F::from(0.0);
                let mut y = F::from(0.0);
                let mut iteration = 0;

                let mut x2 = F::from(0.0);
                let mut y2 = F::from(0.0);
                let mut sum = F::from(0.0);

                while sum < four && iteration < context.poi.limit {
                    y = (x + x) * y + y0;

                    x = x2 - y2 + x0;

                    x2 = x * x;

                    y2 = y * y;

                    sum = x2 + y2;

                    iteration += 1;
                }

                pixels[(pixel_y * context.img_height + pixel_x) as usize] =
                    color_rainbow(iteration, context.poi.limit);
            }
        }
    }
}

// TODO: extract to be a strategy
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

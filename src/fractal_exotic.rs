///! Here are fractal implementations which adoption to Teamplate Floating parameter
///! would take ages, hence we select one f64 and implement FractalFunction trait.
use image::Rgb;
use std::mem;
extern crate crossbeam;
extern crate num_cpus;
use crate::{
    fractals::{color_rainbow, FractalFunction},
    main,
};
use rug::Float;

#[cfg(all(
    target_arch = "x86_64",
    any(target_feature = "avx2", target_feature = "avx512f")
))]
use std::arch::x86_64::*;

/// Uses RUG library for arbitrary Floating precision
pub struct MandelbrotRug;

impl FractalFunction<f64> for MandelbrotRug {
    fn draw(
        &self,
        context: &crate::fractal_builder::Context<f64>,
        id: u32,
        height: u32,
        pixels: &mut [Rgb<u8>],
    ) {
        const BIT_PRECISION: u32 = 64;
        let imgx = context.img_width;
        let imgy = context.img_height;

        let pinhole_center = Float::with_val(BIT_PRECISION, context.poi.pinhole_size / 2.0);
        let pinhole_size = Float::with_val(BIT_PRECISION, context.poi.pinhole_size);

        let origin_x = Float::with_val(BIT_PRECISION, context.poi.origin_x);
        let origin_y = Float::with_val(BIT_PRECISION, context.poi.origin_y);

        let float_four = Float::with_val(BIT_PRECISION, 4.0);

        let center_y_offset = Float::with_val(BIT_PRECISION, &origin_y - &pinhole_center);
        let center_x_offset = Float::with_val(BIT_PRECISION, origin_x - &pinhole_center);

        for pixel_y in 0..height {
            let y_offset = pixel_y + id * height;

            // let y0 = origin_y + (y_offset as f64 / imgy as f64) * pinhole_size
            // - pinhole_center;

            let y0 = Float::with_val(
                BIT_PRECISION,
                &(y_offset as f64 / imgy as f64) * &pinhole_size,
            ) + &center_y_offset;

            // TODO: SLOWER!
            // let y0 = Float::with_val(
            //     BIT_PRECISION,
            //     pinhole_size.mul_add_ref(
            //         &Float::with_val(BIT_PRECISION, y_offset as f64 / imgy as f64),
            //         &origin_y,
            //     ),
            // ) - &pinhole_center;

            for pixel_x in 0..context.img_width {
                let x0 = Float::with_val(
                    BIT_PRECISION,
                    &(pixel_x as f64 / imgx as f64) * &pinhole_size,
                ) + &center_x_offset;

                // TODO: SLOWER!
                // let x0 = Float::with_val(
                //     BIT_PRECISION,
                //     pinhole_size.mul_add_ref(
                //         &Float::with_val(BIT_PRECISION, pixel_x as f64 / imgx as f64),
                //         &origin_x,
                //     ),
                // ) - &pinhole_center;

                let mut x = Float::with_val(BIT_PRECISION, 0.0);
                let mut y = Float::with_val(BIT_PRECISION, 0.0);
                let mut iteration = 0;

                let mut x2 = Float::with_val(BIT_PRECISION, 0.0);
                let mut y2 = Float::with_val(BIT_PRECISION, 0.0);
                let mut sum = Float::with_val(BIT_PRECISION, 0.0);

                while sum < float_four && iteration < context.poi.limit {
                    // y = (x + x) * y + y0;
                    y = y.mul_add(&Float::with_val(BIT_PRECISION, &x + &x), &y0);

                    // x = x2 - y2 + x0;
                    x = Float::with_val(BIT_PRECISION, &x2 - &y2) + &x0;

                    // x2 = x * x;
                    x2 = Float::with_val(BIT_PRECISION, x.square_ref());

                    // y2 = y * y;
                    y2 = Float::with_val(BIT_PRECISION, y.square_ref());

                    // sum = x2 + y2;
                    sum = Float::with_val(BIT_PRECISION, &x2 + &y2);

                    iteration += 1;
                }

                pixels[(pixel_y * context.img_height + pixel_x) as usize] =
                    color_rainbow(iteration, context.poi.limit);
            }
        }
    }
}

/// Uses SIMD AVX2 intrinsic
pub struct MandelbrotAvx2;

impl FractalFunction<f64> for MandelbrotAvx2 {
    fn draw(
        &self,
        context: &crate::fractal_builder::Context<f64>,
        id: u32,
        height: u32,
        pixels: &mut [Rgb<u8>],
    ) {
        if !is_x86_feature_detected!("avx2") {
            panic!("AVX2 not supported on this platform :(");
        }

        let imgx = context.img_width;
        let imgy = context.img_height;

        let pinhole_center = context.poi.pinhole_size / 2.0;
        let x0_offset = context.poi.origin_x - pinhole_center;

        // SIMD part of code
        unsafe {
            // 4 doubles with bin representation of 0xff...
            let ff_mask = _mm256_cmp_pd(_mm256_set1_pd(1.0), _mm256_set1_pd(4.0), _CMP_LE_OQ);

            let mut mask;

            for pixel_y in 0..height {
                let y_offset = pixel_y + id * height;
                let y0 = context.poi.origin_y
                    + (y_offset as f64 / imgy as f64) * context.poi.pinhole_size
                    - pinhole_center;

                let y0 = _mm256_set1_pd(y0);

                // Step by 4, on every iteration we take 4 floats at once
                for pixel_x in (0..context.img_width).step_by(4) {
                    let mut iteration = [0, 0, 0, 0];

                    // let x0 = (pixel_x as f64 / imgx as f64) * self.pinhole_size + x0_offset;
                    // + x0_offset
                    let x0 = _mm256_add_pd(
                        // * self.pinhole_size
                        _mm256_mul_pd(
                            // pixel_x as f64 / imgx as f64
                            _mm256_div_pd(
                                _mm256_set_pd(
                                    (pixel_x + 3) as f64,
                                    (pixel_x + 2) as f64,
                                    (pixel_x + 1) as f64,
                                    pixel_x as f64,
                                ),
                                _mm256_set1_pd(imgx as f64),
                            ),
                            _mm256_set1_pd(context.poi.pinhole_size),
                        ),
                        _mm256_set1_pd(x0_offset),
                    );

                    // let mut x = 0.0;
                    let mut x = _mm256_setzero_pd();
                    // let mut y = 0.0;
                    let mut y = _mm256_setzero_pd();
                    // let mut x2 = 0.0;
                    let mut x2 = _mm256_setzero_pd();
                    // let mut y2 = 0.0;
                    let mut y2 = _mm256_setzero_pd();
                    // let mut sum = 0.0;
                    let mut sum = _mm256_setzero_pd();

                    for i in 0..context.poi.limit {
                        // y = (x + x) * y + y0;
                        // + y0
                        y = _mm256_add_pd(
                            // * y
                            _mm256_mul_pd(
                                // x + x
                                _mm256_add_pd(x, x),
                                y,
                            ),
                            y0,
                        );

                        // x = x2 - y2 + x0;
                        // + x0
                        x = _mm256_add_pd(
                            // x2 - y2
                            _mm256_sub_pd(x2, y2),
                            x0,
                        );

                        // TODO: this is ABS for complex numbers, maybe there is some instrict?

                        // x2 = x * x;
                        x2 = _mm256_mul_pd(x, x);

                        // y2 = y * y;
                        y2 = _mm256_mul_pd(y, y);

                        // sum = x2 + y2;
                        sum = _mm256_add_pd(x2, y2);

                        // iteration += 1;
                        // TODO: other way to unpack?
                        let sum_unpacked: [f64; 4] = mem::transmute(sum);

                        // TODO: this can be SIMD too!
                        //iteration_test =
                        // _mm256_add_pd(_mm256_and_pd(mask, ff_mask), iteration_test);
                        // Returns NaN instead of 1.0 :\
                        //
                        // __m256 mask = _mm256_cmp_ps(mag2, threshold, _CMP_LT_OS);
                        // mk = _mm256_add_ps(_mm256_and_ps(mask, one), mk);
                        //
                        for i in 0..sum_unpacked.len() {
                            iteration[i] = iteration[i] + (sum_unpacked[i] < 4.0) as u32;
                        }

                        // sum < 4.0, _CMP_LE_OQ == Less-than-or-equal (ordered, non-signaling)
                        mask = _mm256_cmp_pd(sum, _mm256_set1_pd(4.0), _CMP_LE_OQ);
                        // Mask will contain 0xfff... if pred is true, 0x000... otherwise

                        // If mask is all 0, all points in the vector escaped 4.0 circle, break the loop
                        if _mm256_testz_pd(mask, ff_mask) == 1 {
                            break;
                        }
                    }

                    for i in 0..4 {
                        pixels[(pixel_y * context.img_height + pixel_x + i) as usize] =
                            color_rainbow(iteration[i as usize], context.poi.limit);
                    }
                }
            }
        } // unsafe
    }
}

/// Uses SIMD AVX512 intrinsic
pub struct MandelbrotAvx512;

impl FractalFunction<f64> for MandelbrotAvx512 {
    fn draw(
        &self,
        context: &crate::fractal_builder::Context<f64>,
        id: u32,
        height: u32,
        pixels: &mut [Rgb<u8>],
    ) {
        if !is_x86_feature_detected!("avx512f") {
            panic!("avx512f not supported on this platform :(");
        }

        let imgx = context.img_width;
        let imgy = context.img_height;

        let pinhole_center = context.poi.pinhole_size / 2.0;
        let x0_offset = context.poi.origin_x - pinhole_center;

        // SIMD part of code
        unsafe {
            // 4 doubles with bin representation of 0xff...
            let ff_mask = _mm512_cmp_pd_mask(_mm512_set1_pd(1.0), _mm512_set1_pd(4.0), _CMP_LE_OQ);

            for pixel_y in 0..height {
                let y_offset = pixel_y + id * height;
                let y0 = context.poi.origin_y
                    + (y_offset as f64 / imgy as f64) * context.poi.pinhole_size
                    - pinhole_center;

                let y0 = _mm512_set1_pd(y0);

                // Step by 8, on every iteration we take 8 floats at once
                for pixel_x in (0..context.img_width).step_by(8) {
                    let mut iteration = [0, 0, 0, 0, 0, 0, 0, 0];

                    // let x0 = (pixel_x as f64 / imgx as f64) * self.pinhole_size + x0_offset;
                    // + x0_offset
                    let x0 = _mm512_add_pd(
                        // * self.pinhole_size
                        _mm512_mul_pd(
                            // pixel_x as f64 / imgx as f64
                            _mm512_div_pd(
                                _mm512_set_pd(
                                    (pixel_x + 7) as f64,
                                    (pixel_x + 6) as f64,
                                    (pixel_x + 5) as f64,
                                    (pixel_x + 4) as f64,
                                    (pixel_x + 3) as f64,
                                    (pixel_x + 2) as f64,
                                    (pixel_x + 1) as f64,
                                    pixel_x as f64,
                                ),
                                _mm512_set1_pd(imgx as f64),
                            ),
                            _mm512_set1_pd(context.poi.pinhole_size),
                        ),
                        _mm512_set1_pd(x0_offset),
                    );

                    // let mut x = 0.0;
                    let mut x = _mm512_setzero_pd();
                    // let mut y = 0.0;
                    let mut y = _mm512_setzero_pd();
                    // let mut x2 = 0.0;
                    let mut x2 = _mm512_setzero_pd();
                    // let mut y2 = 0.0;
                    let mut y2 = _mm512_setzero_pd();
                    // let mut sum = 0.0;
                    let mut sum = _mm512_setzero_pd();

                    // TODO: try to change to range loop, should be no difference
                    let mut i = 0;
                    while i < context.poi.limit {
                        i += 1;

                        // y = (x + x) * y + y0;
                        // + y0
                        y = _mm512_add_pd(
                            // * y
                            _mm512_mul_pd(
                                // x + x
                                _mm512_add_pd(x, x),
                                y,
                            ),
                            y0,
                        );

                        // x = x2 - y2 + x0;
                        // + x0
                        x = _mm512_add_pd(
                            // x2 - y2
                            _mm512_sub_pd(x2, y2),
                            x0,
                        );

                        // TODO: this is ABS for complex numbers, maybe there is some instrict?

                        // x2 = x * x;
                        x2 = _mm512_mul_pd(x, x);

                        // y2 = y * y;
                        y2 = _mm512_mul_pd(y, y);

                        // sum = x2 + y2;
                        sum = _mm512_add_pd(x2, y2);

                        // iteration += 1;
                        // TODO: other way to unpack?
                        let sum_unpacked: [f64; 8] = mem::transmute(sum);

                        // TODO: this can be SIMD too!
                        //iteration_test =
                        // _mm512_add_pd(_mm512_and_pd(mask, ff_mask), iteration_test);
                        // Returns NaN instead of 1.0 :\
                        for i in 0..sum_unpacked.len() {
                            iteration[i] = iteration[i] + (sum_unpacked[i] < 4.0) as u32;
                        }

                        // sum < 4.0, _CMP_LE_OQ == Less-than-or-equal (ordered, non-signaling)
                        let mask = _mm512_cmp_pd_mask(sum, _mm512_set1_pd(4.0), _CMP_LE_OQ);
                        // Mask will contain 0x1 per element if pred is true

                        // If mask is all 0, all points in the vector escaped 4.0 circle, break the loop
                        if mask == 0 {
                            break;
                        }
                    }

                    for i in 0..8 {
                        pixels[(pixel_y * context.img_height + pixel_x + i) as usize] =
                            color_rainbow(iteration[i as usize], context.poi.limit);
                    }
                }
            } // unsafe
        }
    }
}

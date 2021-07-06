#![feature(stdsimd)]
use fractal::Fractal;

#[macro_use]
extern crate glium;

#[macro_use]
extern crate lazy_static;

mod fractal;
mod opengl;
mod quadruple;

fn main() {
    let fractal = Fractal {
        // For sake of simplicity, keep plane dimensions aligned to 64 bits
        img_width: 608,
        img_height: 608,
        origin_x: 0.0,
        origin_y: 0.0,
        pinhole_size: 4.0,
        pinhole_step: 1.0,
        limit: 200,

    };

    // let pipe = fractal.run_on_thread();
    // let pipe = fractal.run_on_all_cpus_1();
    let pipe = fractal.run_on_rayon();
    // let pipe = fractal.run_on_thread_simd();
    // let pipe = fractal.run_on_rayon_simd();

    opengl::run(pipe);
}

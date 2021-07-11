#![feature(stdsimd)]
use fractal::Fractal;

#[macro_use]
extern crate glium;

#[macro_use]
extern crate lazy_static;

mod fractal;
mod opengl;
mod quadruple;
mod soft_float;

fn main() {
    let fractal = Fractal {
        // For sake of simplicity, keep plane dimensions aligned to 64 bits
        img_width: 608,
        img_height: 608,
        origin_x: -0.7436438870371587,
        origin_y: 0.13182590420531198,
        pinhole_size: 0.0000000000004892965009859402,
        pinhole_step: 1.0,
        limit: 3800,
    
    };

    // let pipe = fractal.run_on_thread();
    // let pipe = fractal.run_on_all_cpus_1();
    let pipe = fractal.run_on_rayon();
    // let pipe = fractal.run_on_thread_simd();
    // let pipe = fractal.run_on_rayon_simd();

    opengl::run(pipe);
}

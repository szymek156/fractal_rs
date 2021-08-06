#![feature(trait_alias)]
#![feature(stdsimd)]
use fractal::Fractal;

#[macro_use]
extern crate glium;

#[macro_use]
extern crate lazy_static;

mod fractal;
mod fractal_builder;
mod opengl;
mod executor;
mod fractals;
mod quadruple;
mod soft_float;

fn main() {
    let fractal = Fractal {
        // For sake of simplicity, keep plane dimensions aligned to 64 bits
        // img_width: 608,
        // img_height: 608,
        // origin_x: -0.7436438870371587,
        // origin_y: 0.13182590420531198,
        // pinhole_size: 0.0000000000004892965009859402,
        // pinhole_step: 1.0,
        // limit: 3800,
        // img_width: 608,
        // img_height: 608,
        // origin_x: -0.7436438870371587,
        // origin_y: 0.13182590420531198,
        // pinhole_size: 2.0,
        // pinhole_step: 1.0,
        // limit: 200,

        // Zoom on quad
        // img_width: 608,
        // img_height: 608,
        // origin_x: -0.7436438870371579,
        // origin_y: 0.13182590420531123,
        // pinhole_size: 0.000000000000000006216137609703,
        // pinhole_step: 0.9,
        // limit: 6200,


        // img_width: 608,
        // img_height: 608,
        // origin_x: -1.629162717619015,
        // origin_y: -0.02022437964772109,
        // pinhole_size: 0.000000000000008289098145403582,
        // pinhole_step: 1.0,
        // limit: 600,
        img_width: 608,
        img_height: 608,
        origin_x: -1.275160031112145,
        origin_y: -0.19410769865119987,
        pinhole_size: 0.000000000000000038614262509059454,
        pinhole_step: 1.0,
        limit: 1200,

    };

    // TODO: Interesting, artifacts on quadruple implementation
    // img_width: 608,
    // img_height: 608,
    // origin_x: -1.275160031112145,
    // origin_y: -0.19410769865119987,
    // pinhole_size: 0.000000000000000038614262509059454,
    // pinhole_step: 1.0,
    // limit: 1200,

    // let pipe = fractal.run_on_thread();
    // let pipe = fractal.run_on_all_cpus_1();
    let pipe = fractal.run_on_rayon();
    // let pipe = fractal.run_on_thread_simd();
    // let pipe = fractal.run_on_rayon_simd();

    opengl::run(pipe);
}

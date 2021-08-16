#![feature(trait_alias)]
#![feature(stdsimd)]
// use fractal::Fractal;

use quadruple::Quad;

use crate::{executor::ExecutorKind, fractal_builder::Fractal, fractals::PoI};
use crate::fractal::Fractal as OldFractal;

#[macro_use]
extern crate glium;

#[macro_use]
extern crate lazy_static;

mod executor;
mod fractal;
mod fractal_builder;
mod fractals;
mod opengl;
mod quadruple;
mod soft_float;
mod pipe;


fn run_old() {
    let fractal = OldFractal {
        // For sake of simplicity, keep plane dimensions aligned to 64 bits
        img_width: 608,
        img_height: 608,
        origin_x: -0.7436438870371587,
        origin_y: 0.13182590420531198,
        pinhole_size: 0.0000000000004892965009859402,
        pinhole_step: 1.0,
        limit: 3800,
    //     // img_width: 608,
    //     // img_height: 608,
    //     // origin_x: -0.7436438870371587,
    //     // origin_y: 0.13182590420531198,
    //     // pinhole_size: 2.0,
    //     // pinhole_step: 1.0,
    //     // limit: 200,

    //     // Zoom on quad
    //     // img_width: 608,
    //     // img_height: 608,
    //     // origin_x: -0.7436438870371579,
    //     // origin_y: 0.13182590420531123,
    //     // pinhole_size: 0.000000000000000006216137609703,
    //     // pinhole_step: 0.9,
    //     // limit: 6200,

    //     // img_width: 608,
    //     // img_height: 608,
    //     // origin_x: -1.629162717619015,
    //     // origin_y: -0.02022437964772109,
    //     // pinhole_size: 0.000000000000008289098145403582,
    //     // pinhole_step: 1.0,
    //     // limit: 600,
    //     img_width: 608,
    //     img_height: 608,
    //     origin_x: -1.275160031112145,
    //     origin_y: -0.19410769865119987,
    //     pinhole_size: 0.000000000000000038614262509059454,
    //     pinhole_step: 1.0,
    //     limit: 1200,

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
fn main() {

    // run_old();
    
    let mut fractal = Fractal::<Quad>::default()
        .mandelbrot()
        .with_poi(PoI { // Template type deduction!
            // origin_x: Quad::from(-1.275160031112145),
            // origin_y: Quad::from(-0.19410769865119987),
            // pinhole_size: Quad::from(0.000000000000000038614262509059454),
            // limit: 1200,

            // Limit of Quad, yay!
            origin_x: Quad {
                lo: 0.000000000000000010150844351198857,
                hi: -1.275160031112145,
            },
            origin_y: Quad {
                lo: 0.000000000000000006705298387715869,
                hi: -0.19410769865119987,
            },
            pinhole_size: Quad {
                lo: 0.00000000000000000000000000000000000000000000002084738436862751,
                hi: 0.0000000000000000000000000000004272805497045327,
            },
            limit: 1400,
        
            // origin_x: -0.7436438870371587,
            // origin_y: 0.13182590420531198,
            // pinhole_size: 0.0000000000004892965009859402,
            // limit: 3800,
        })
        .run_on(ExecutorKind::Rayon);

    let pipe = fractal.start();    
    opengl::run(pipe);
}

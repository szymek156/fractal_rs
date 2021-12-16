#![feature(trait_alias)]
#![feature(stdsimd)]

use std::marker::PhantomData;

use fractal_exotic::{MandelbrotAvx2, MandelbrotRug};
use fractals::Mandelbrot;
use quadruple::Quad;

use crate::{executor::ExecutorKind, fractal_builder::Fractal, fractals::PoI};

#[macro_use]
extern crate glium;

#[macro_use]
extern crate lazy_static;

mod executor;
mod fractal_builder;
mod fractal_exotic;
mod fractals;
mod opengl;
mod pipe;
mod quadruple;
mod soft_float;

fn main() {
    let mut fractal =
            // Fractal::<Quad>::default()
            // .with(Box::new(Mandelbrot(PhantomData)))
            // .with_poi(PoI {
            //     // Limit of Quad, yay!
            //     origin_x: Quad {
            //         lo: 0.000000000000000010150844351198857,
            //         hi: -1.275160031112145,
            //     },
            //     origin_y: Quad {
            //         lo: 0.000000000000000006705298387715869,
            //         hi: -0.19410769865119987,
            //     },
            //     pinhole_size: Quad {
            //         lo: 0.00000000000000000000000000000000000000000000002084738436862751,
            //         hi: 0.0000000000000000000000000000004272805497045327,
            //     },
            //     limit: 1400,
            // })
        Fractal::<f64>::default()
        .with(Box::new(MandelbrotAvx2))
        .with_poi(PoI {
            // Template type deduction!
            // origin_x: -1.275160031112145,
            // origin_y: -0.19410769865119987,
            // pinhole_size: 0.000000000000000038614262509059454,
            // limit: 1200,
            origin_x: -0.7436438870371587,
            origin_y: 0.13182590420531198,
            pinhole_size: 0.0000000000004892965009859402,
            limit: 3800,
        })
        .run_on(ExecutorKind::Rayon);

    let pipe = fractal.start();
    opengl::run(pipe);
}

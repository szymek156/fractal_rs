use fractal::Fractal;

#[macro_use]
extern crate glium;

#[macro_use]
extern crate lazy_static;

mod fractal;
mod opengl;

fn main() {
    let fractal = Fractal {
        // For sake of simplicity, keep plane dimensions aligned to 64 bits
        img_width: 608,
        img_height: 608,
        origin_x: -1.256884046123662,
        origin_y: 0.3796264149022917,
        pinhole_size: 0.00000000001004489644531639,
        pinhole_step: 1.0,
        limit: 1200,

    };

    let pipe = fractal.run_on_thread_simd();
    // let pipe = fractal.run_on_thread();
    // let pipe = fractal.run_on_all_cpus_2();

    opengl::run(pipe);
}


use fractal::Fractal;

#[macro_use]
extern crate glium;

#[macro_use]
extern crate lazy_static;

mod fractal;
mod opengl;

fn main() {
    let fractal = Fractal {
        img_height: 800,
        img_width: 800,
        origin_x: 0.0,
        origin_y: 0.0,
        pinhole_size: 4.0,
        pinhole_step: 1.0,
        limit: 500
    };

    let pipe = fractal.run_on_all_cpus_2();
    opengl::run(pipe);
}

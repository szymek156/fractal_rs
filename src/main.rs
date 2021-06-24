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
        origin_x: -1.2583384664947936,
        origin_y: -0.032317669198187016,
        pinhole_size: 4.0,
        pinhole_step: 1.0,
        limit: 200
    };

    let pipe = fractal.run_on_thread();
    opengl::run(pipe);
}

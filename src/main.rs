
#[macro_use]
extern crate glium;


#[macro_use]
extern crate lazy_static;

mod opengl;
mod fractal;

fn main() {
    let receiver = fractal::run_on_thread();
    opengl::run(receiver);
}

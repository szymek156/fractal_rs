use crate::{
    executor::{Executor, ExecutorKind, Rayon},
    fractals::{Floating, FractalFunction, Mandelbrot, PoI},
    pipe::Pipe,
};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Context<F> {
    pub img_width: u32,
    pub img_height: u32,

    pub pinhole_step: F,
    pub poi: PoI<F>,
}
pub struct Fractal<F: Floating> {
    context: Context<F>,
    fractal_function: Box<dyn FractalFunction<F>>,
    executor: Box<dyn Executor<F>>,
}

impl<F: Floating> Default for Fractal<F> {
    fn default() -> Self {
        Fractal {
            context: Context {
                img_height: 608,
                img_width: 608,
                pinhole_step: F::from(1.0),
                poi: PoI {
                    origin_x: F::from(0.0),
                    origin_y: F::from(0.0),
                    pinhole_size: F::from(4.0),
                    limit: 300,
                },
            },
            fractal_function: Box::new(Mandelbrot::<F>(PhantomData)),
            executor: Box::new(Rayon),
        }
    }
}

impl<F: Floating> Fractal<F> {
    pub fn mandelbrot(mut self) -> Self {
        self.fractal_function = Box::new(Mandelbrot::<F>(PhantomData));

        self
    }

    pub fn julia(mut self) -> Self {
        todo!();
    }

    /// This time use enum, because... why not
    pub fn run_on(mut self, executor: ExecutorKind) -> Self {
        match executor {
            ExecutorKind::SingleThread => todo!(),
            ExecutorKind::Rayon => self.executor = Box::new(Rayon),
        }

        self
    }

    pub fn with_poi(mut self, poi: PoI<F>) -> Self {
        self.context.poi = poi;

        self
    }

    pub fn start(self) -> Pipe {
        self.executor.execute(self.context, self.fractal_function)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn using_builder_pattern() {
        let mut fractal = Fractal::<f64>::default()
            .mandelbrot()
            .with_poi(PoI::<f64> {
                origin_x: -1.275160031112145,
                origin_y: -0.19410769865119987,
                pinhole_size: 0.000000000000000038614262509059454,
                limit: 1200,
            })
            .run_on(ExecutorKind::Rayon);

        let _pipe = fractal.start();
    }
}

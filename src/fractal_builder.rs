use crate::{executor::{Executor, ExecutorType, Rayon}, fractals::{Floating, FractalFunction, Mandelbrot, PoI}};
use std::marker::PhantomData;

// #[derive(Debug)]
pub struct Fractal<F: Floating> {
    pub img_width: u32,
    pub img_height: u32,

    pub pinhole_step: F,
    pub poi: PoI<F>,
    fractal_function: Box<dyn FractalFunction<F>>,
    executor: Box<dyn Executor>
}

impl<F: Floating> Default for Fractal<F> {
    fn default() -> Self {
        Fractal {
            img_height: 608,
            img_width: 608,
            pinhole_step: F::from(1.0),
            poi: PoI {
                origin_x: F::from(0.0),
                origin_y: F::from(0.0),
                pinhole_size: F::from(4.0),
                limit: 300,
            },
            fractal_function: Box::new(Mandelbrot::<F>(PhantomData)),
            executor: Box::new(Rayon)
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
    pub fn run_on(mut self, executor: ExecutorType) -> Self {
        match  executor {
            ExecutorType::SingleThread => todo!(),
            ExecutorType::Rayon => self.executor = Box::new(Rayon),
        }

        self
     }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn using_builder_pattern() {
        let _fractal = Fractal::<f64>::default().mandelbrot();
    }
}

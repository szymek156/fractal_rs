use std::{marker::PhantomData, rc::Rc};


trait Floating = From<f64> + Copy + 'static;

#[derive(Debug, Default)]
pub struct PoI<Floating> {
    pub origin_x: Floating,
    pub origin_y: Floating,
    pub pinhole_size: Floating,
    pub limit: u32,
}

// #[derive(Debug)]
pub struct Fractal<Floating> {
    pub img_width: u32,
    pub img_height: u32,

    pub pinhole_step: Floating,
    pub poi: PoI<Floating>,
    pub fractal_function: Box<dyn FractalFunction>,
}

impl<F : Floating> Default for Fractal<F>

{
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
            fractal_function: Box::new(Mandelbrot::<F>{poi: None}),
        }
    }
}

trait FractalFunction {
    // &self to have safe object
    fn draw(&self);
}

// Unused type parameters cause some internal compiler problems
// that I do not understand, and they were made illegal long time
// ago. _marker is zero sized type, that pretends usage of Floating,
// making compiler happy.
struct Mandelbrot<F: Floating> {
    pub poi : Option<PoI<F>>
}

impl<F : Floating> FractalFunction for Mandelbrot<F>
{
    fn draw(&self) {
        let poi = self.poi.as_ref().unwrap();
        let sample = poi.origin_x; //+ Floating::from(2.0);

        let sample = F::from(6.9);
        todo!();
    }
}

impl<F: Floating> Fractal<F>
{
    pub fn mandelbrot(mut self) -> Self {
        
        self.fractal_function = Box::new(Mandelbrot::<F>{poi: None});

        self
    }

    

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn using_builder_pattern() {
        let fractal = Fractal::<f64>::default().mandelbrot();
    }
}

use std::marker::PhantomData;


/// Trait defining underlying floating type
pub trait Floating = From<f64> + Copy + 'static;

/// PoI - point of interest on a complex plane
#[derive(Debug, Default)]
pub struct PoI<Floating> {
    pub origin_x: Floating,
    pub origin_y: Floating,
    pub pinhole_size: Floating,
    pub limit: u32,
}

/// Interface required for fractal to be implemented for drawing purposes
pub trait FractalFunction<F:Floating> {
    // &self to have safe object
    fn draw(&self, poi : &PoI<F>);
}

// Unused type parameters cause some internal compiler problems
// that I do not understand, and they were made illegal long time
// ago. _marker is zero sized type, that pretends usage of Floating,
// making compiler happy.
pub struct Mandelbrot<F: Floating> (pub PhantomData<F>);

impl<F: Floating> FractalFunction<F> for Mandelbrot<F> {
    fn draw(&self, poi : &PoI<F>) {
        let _sample = poi.origin_x; //+ Floating::from(2.0);

        let _sample = F::from(6.9);
        todo!();
    }
}
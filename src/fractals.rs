use std::{fmt::Debug, marker::PhantomData, ops::{Add, AddAssign, Div, Mul, MulAssign, Sub, SubAssign}};

/// Trait defining underlying floating type
// Send to safely pass type over threads
// TODO: maybe there is already numeric trait, which fulfills all those?
// Mul<Output=Self> means, Type has to implement Mul, and result of this operation also needs to be Floating
// Thats not always the case - see Rug implementation
pub trait Floating = From<f64>
    + Copy
    + 'static
    + MulAssign
    + Mul<Output = Self>
    + Div<Output = Self>
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Send
    + Debug;

/// PoI - point of interest on a complex plane
#[derive(Debug, Default)]
pub struct PoI<Floating> {
    pub origin_x: Floating,
    pub origin_y: Floating,
    pub pinhole_size: Floating,
    pub limit: u32,
}

/// Interface required for fractal to be implemented for drawing purposes
pub trait FractalFunction<F: Floating> {
    // &self to have safe object
    fn draw(&self, poi: &PoI<F>);
}

// Unused type parameters cause some internal compiler problems
// that I do not understand, and they were made illegal long time
// ago. _marker is zero sized type, that pretends usage of Floating,
// making compiler happy.
pub struct Mandelbrot<F: Floating>(pub PhantomData<F>);

impl<F: Floating> FractalFunction<F> for Mandelbrot<F> {
    fn draw(&self, poi: &PoI<F>) {
        let _sample = poi.origin_x + F::from(2.0);

        let _sample = F::from(6.9);
        todo!();
    }
}

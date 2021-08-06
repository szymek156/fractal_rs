use std::marker::PhantomData;

#[derive(Debug)]
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

impl<Floating> Default for Fractal<Floating>
where
    Floating: From<f64> + 'static
{
    fn default() -> Self {
        Fractal {
            img_height: 608,
            img_width: 608,
            pinhole_step: Floating::from(1.0),
            poi: PoI {
                origin_x: Floating::from(0.0),
                origin_y: Floating::from(0.0),
                pinhole_size: Floating::from(4.0),
                limit: 300,
            },
            fractal_function: Box::new(Mandelbrot::<Floating>(PhantomData)),
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
struct Mandelbrot<Floating>(PhantomData<Floating>);

impl<Floating> FractalFunction for Mandelbrot<Floating>
where
    Floating: From<f64>,
{
    fn draw(&self) {
        let sample = Floating::from(6.9);
        todo!();
    }
}

impl<Floating> Fractal<Floating>
where
    Floating: From<f64> + 'static
{
    pub fn mandelbrot(mut self) -> Self {
        
        self.fractal_function = Box::new(Mandelbrot::<Floating>(PhantomData));
        // Box::new(Mandelbrot {
        //     _marker: PhantomData::<Floating>,
        // });

        self
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_fr() {
        let m: Box<dyn FractalFunction> = Box::new(Mandelbrot::<f64>(PhantomData));
    }
    #[test]
    fn using_builder_pattern() {
        let fractal = Fractal::<f64>::default().mandelbrot();
    }
}

use std::marker::PhantomData;

#[derive(Debug)]
pub struct PoI<Floating> {
    pub origin_x: Floating,
    pub origin_y: Floating,
    pub pinhole_size: Floating,
    pub limit: u32,
}


// Enum approach:
// - Still unused type parameter, 'phantom' data is hidden in Julia(T)
// - It also has to be initialized: Julia(Floating::from(4.4))
// but contrary to phantom - it takes place
// - But phantom can be used so point above may be irrelevant 
// - match inside draw method has to be there - dispatching done by hand
// - no heap allocation
pub enum FractalEnum<T> {
    NotSet,
    Julia(T),
    Mandelbrot(T),
}

impl<T> FractalEnum<T>
where
    T: From<f64>,
{
    fn draw(&self) {
        match self {
            FractalEnum::NotSet => todo!(),
            FractalEnum::Julia(_) => todo!(),
            FractalEnum::Mandelbrot(_) => FractalEnum::<T>::mandelbrot(),
        }
    }

    fn mandelbrot() {
        let some_var = T::from(4.0);
        todo!()
    }
}

// #[derive(Debug)]
pub struct Fractal<Floating> {
    pub img_width: u32,
    pub img_height: u32,

    pub pinhole_step: Floating,
    pub poi: PoI<Floating>,
    pub fractal_function: Box<dyn FractalFunction>,
    pub fractal_enum: FractalEnum<Floating>,
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
            fractal_enum: FractalEnum::NotSet,
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

    pub fn mandelbrot_enum(mut self) -> Self {
        self.fractal_enum = FractalEnum::Mandelbrot(Floating::from(6.9));

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
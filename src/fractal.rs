use image::{ImageBuffer, Rgb};
use num_complex::Complex;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Instant;

pub type OutBuffer = ImageBuffer<Rgb<u8>, Vec<u8>>;

pub enum Command {
    ZoomOut,
    ZoomIn,
    LessIterations,
    MoreIterations,
    ChangeOrigin(f64, f64),
}
pub struct Pipe {
    pub img_rcv: Receiver<OutBuffer>,
    pub cmd_send: Sender<Command>,
}

pub struct Fractal {
    pub img_width: u32,
    pub img_height: u32,
    pub origin_x: f64,
    pub origin_y: f64,
    pub pinhole_size: f64,
    pub pinhole_step: f64,
    pub limit: u32,
}

impl Fractal {
    fn handle_command(&mut self, command: Command) {
        match command {
            Command::ZoomOut => self.pinhole_step += 0.1,
            Command::ZoomIn => self.pinhole_step -= 0.1,
            Command::LessIterations => self.limit -= 200.max(self.limit - 200),
            Command::MoreIterations => self.limit += 200,
            Command::ChangeOrigin(x, y) => {
                let pinhole_center = self.pinhole_size / 2.0;

                self.origin_x = self.origin_x + ((x / self.img_width as f64) * self.pinhole_size)
                    - pinhole_center;
                self.origin_y = self.origin_y + ((y / self.img_height as f64) * self.pinhole_size)
                    - pinhole_center;
            }
        }
    }
    pub fn simple_julia(&self) -> OutBuffer {
        // Oh lol:
        // https://crates.io/crates/image
        let imgx = self.img_width;
        let imgy = self.img_height;

        let scalex = self.pinhole_size / imgx as f64;
        let scaley = self.pinhole_size / imgy as f64;

        // Create a new ImgBuf with width: imgx and height: imgy
        let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

        // Iterate over the coordinates and pixels of the image
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let r = (0.3 * x as f64) as u8;
            let b = (0.3 * y as f64) as u8;
            *pixel = image::Rgb([r, 0, b]);

            let cx = y as f64 * scalex - 1.5;
            let cy = x as f64 * scaley - 1.5;

            let c = Complex::new(-0.4, 0.6);
            let mut z = Complex::new(cx, cy);

            let mut i = 0;
            while i < self.limit && z.norm() <= 2.0 {
                z = z * z + c;
                i += 1;
            }

            let image::Rgb(data) = *pixel;
            *pixel = image::Rgb([data[0], i as u8, data[2]]);
        }

        imgbuf
    }

    pub fn mandelbrot(&self) -> OutBuffer {
        let imgx = self.img_width;
        let imgy = self.img_height;

        let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let pinhole_center = self.pinhole_size / 2.0;
            let x0 = self.origin_x + (x as f64 / imgx as f64) * self.pinhole_size - pinhole_center;
            let y0 = self.origin_y + (y as f64 / imgy as f64) * self.pinhole_size - pinhole_center;

            let mut x = 0.0;
            let mut y = 0.0;
            let mut iteration = 0;

            let mut x2 = 0.0;
            let mut y2 = 0.0;
            let mut sum = 0.0;

            while sum < 4.0 && iteration < self.limit {
                y = (x + x) * y + y0;

                x = x2 - y2 + x0;

                x2 = x * x;

                y2 = y * y;

                sum = x2 + y2;

                iteration += 1;
            }

            *pixel = color_rainbow(iteration, self.limit);
        }

        imgbuf
    }

    pub fn run_on_thread(mut self) -> Pipe {
        let (img_send, img_rcv) = channel();

        let (cmd_send, cmd_rcv) = channel();

        let pipe = Pipe {
            cmd_send: cmd_send,
            img_rcv: img_rcv,
        };

        thread::spawn(move || loop {
            match cmd_rcv.try_recv() {
                Ok(command) => {
                    println!("Got command!");
                    self.handle_command(command)
                }
                Err(_) => (),
            }

            let start = Instant::now();

            let image = self.mandelbrot();

            println!("Render took {}", start.elapsed().as_millis());

            img_send.send(image).unwrap();

            self.pinhole_size *= self.pinhole_step;
        });

        pipe
    }
}

fn color_gray(iteration: u32, limit: u32) -> image::Rgb<u8> {
    let lum = (iteration as f32 / limit as f32 * 255.0) as u8;
    image::Rgb([lum, lum, lum])
}

fn color_rainbow(iteration: u32, limit: u32) -> image::Rgb<u8> {
    // TODO: variable names are nonsense, refactor

    let mut pixel = image::Rgb([0, 0, 0]);

    if iteration < limit {
        let mut h = iteration as f64 % 360.0;
        h /= 60.0;
        let i = h as usize;
        let mut f = h - i as f64; // factorial part of h
        let mut q = 1.0 - f;

        f *= 255.0;
        q *= 255.0;

        let r_arr = [255, q as u8, 0, 0, f as u8, 255];
        let g_arr = [f as u8, 255, 255, q as u8, 0, 0];
        let b_arr = [0, 0, f as u8, 255, 255, q as u8];

        pixel = image::Rgb([r_arr[i], g_arr[i], b_arr[i]])
    }

    pixel
}

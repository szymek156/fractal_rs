use image::{ImageBuffer, Rgb};
use num_complex::Complex;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Instant;
extern crate crossbeam;
extern crate num_cpus;
use crossbeam::thread::scope;
use std::sync::{Arc, Mutex};

pub type OutBuffer = ImageBuffer<Rgb<u8>, Vec<u8>>;
#[derive(Debug)]
pub enum Command {
    ZoomOut,
    ZoomIn,
    LessIterations,
    MoreIterations,
    ChangeOrigin(f64, f64),
    SetPOI(u32),
    GetState,
}
pub struct Pipe {
    pub img_rcv: Receiver<OutBuffer>,
    pub cmd_send: Sender<Command>,
}
#[derive(Debug, Clone)]
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

                // * -1.0 because Y values increase in down direction
                self.origin_y = self.origin_y
                    + (((y / self.img_height as f64) * self.pinhole_size) - pinhole_center) * -1.0;
            }
            Command::SetPOI(poi) => match poi {
                0 => {
                    self.origin_x = 0.0;
                    self.origin_y = 0.0;
                    self.pinhole_size = 4.0;
                    self.pinhole_step = 1.0;
                    self.limit = 20;
                }
                1 => {
                    self.origin_x = -1.2583384664947936;
                    self.origin_y = -0.032317669198187016
                }
                2 => {
                    self.origin_x = -1.2487780999747029;
                    self.origin_y = 0.071802096973029209;
                }
                3 => {
                    self.origin_x = -1.2583385189936513;
                    self.origin_y = -0.032317635405726151;
                }
                4 => {
                    self.origin_x = -1.2583384664947908;
                    self.origin_y = -0.032317669198180785;
                }
                5 => {
                    self.origin_x = -1.4780998580724920;
                    self.origin_y = -0.0029962325962097328;
                }
                6 => {
                    self.origin_x = 0.3994999999000;
                    self.origin_y = -0.195303;
                }
                7 => {
                    self.origin_x = -1.768611136076306;
                    self.origin_y = -0.001266863985331;
                }
                8 => {
                    self.origin_x = -1.7686112281079116;
                    self.origin_y = -0.0012668963162883458;
                }
                9 => {
                    self.origin_x = -1.2568840461035797;
                    self.origin_y = 0.3796264149862358;
                }
                _ => (),
            },
            Command::GetState => {
                println!("Current position: {:#?}", self);
                println!("Zoom: {}", 4.0 / self.pinhole_size);
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

    pub fn mandelbrot(&self, imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
        let imgx = self.img_width;
        let imgy = self.img_height;

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
    }

    // TODO: multithread manually
    // TODO: multithread rayon
    // TODO: cuda wrapper?
    // TODO: SIMD

    pub fn run_on_all_cpus(mut self) -> Pipe {
        let (img_send, img_rcv) = channel();

        let (cmd_send, cmd_rcv) = channel();

        let pipe = Pipe {
            cmd_send: cmd_send,
            img_rcv: img_rcv,
        };

        // Approach one, in order to share common state, use Arc + Mutex, and do a self.clone()
        // to avoid complaining that threads outlives self.
        thread::spawn(move || {
            let num_threads = num_cpus::get();

            // Share a context among threads, in order to do so,
            // Make a clone of self (to avoid complaining that closures inside spawn outlives self)
            // Wrap it in a mutex (to have safe access to context)
            // Wrap into Arc (to have possibility to share it among threads - mutex does not have clone!)
            let mutex = Arc::new(Mutex::new(self.clone()));

            // TODO: unsafe cell?
            let img_buff = image::ImageBuffer::new(self.img_width, self.img_height);

            // let sub_img = image::SubImage::new(img_buff, 0, 0, self.img_width, 200);


            let mut threads = vec![];
            for _ in 0..num_threads {
                let mutex = mutex.clone();
                let mut img_buff = img_buff.clone();
                threads.push(thread::spawn(move || {
                    let context = mutex.lock().unwrap().clone();
                    context.mandelbrot(&mut img_buff);
                }));
            }

            loop {
                match cmd_rcv.recv() {
                    Ok(command) => {
                        let mut context = mutex.lock().unwrap();
                        println!("Got command {:?}!", command);
                        context.handle_command(command)
                    }
                    Err(_) => break,
                }
            }
        });

        // Approach two, use crossbeam::scope
        // thread::spawn(move || {
        //     let num_threads = num_cpus::get();
        //     let img_buff = image::ImageBuffer::new(self.img_width, self.img_height);

        //     crossbeam::scope(|s| {
        //         let mut threads = vec![];
        //         for i in 0..num_threads {
        //             threads.push(s.spawn(|_| {
        //                 // let i = i;s
        //                 self.mandelbrot(&mut img_buff.clone());

        //                 println!("Thread ends");
        //             }));
        //         }
        //     }).unwrap();

        //     // All threads joined here already, so nope, crossbeam not for this scenario :(
        //     loop {
        //         println!("listening on threads");
        //         match cmd_rcv.try_recv() {
        //             Ok(command) => {
        //                 println!("Got command {:?}!", command);
        //                 self.handle_command(command)
        //             }
        //             Err(_) => break,
        //         }
        //     }
        // });

        pipe
    }

    //TODO:  self and mutex does not live long enough, figure out why
    // pub fn run_on_all_cpus(mut self) -> Pipe {
    //     let (img_send, img_rcv) = channel();

    //     let (cmd_send, cmd_rcv) = channel();

    //     let pipe = Pipe {
    //         cmd_send: cmd_send,
    //         img_rcv: img_rcv,
    //     };

    //     thread::spawn(move || {
    //         let num_threads = num_cpus::get();

    //         let mut image = image::ImageBuffer::new(self.img_width, self.img_height);

    //         let mutex = Mutex::new(&mut self);
    //         let threads: Vec<_> = (0..num_threads)
    //             .map(|_| {
    //                 thread::spawn(|| {
    //                     let context = mutex.lock().unwrap().clone();
    //                     context.mandelbrot(&mut image.clone());
    //                 })
    //             })
    //             .collect();

    //         loop {
    //             match cmd_rcv.try_recv() {
    //                 Ok(command) => {
    //                     let mut context = mutex.lock().unwrap();
    //                     println!("Got command {:?}!", command);
    //                     context.handle_command(command)
    //                 }
    //                 Err(_) => break,
    //             }
    //         }

    //         drop(threads);

    //         // let _: Vec<_> = threads
    //         //     .into_iter()
    //         //     .map(|handle| {
    //         //         handle.join().unwrap();
    //         //     })
    //         //     .collect();
    //     });

    //     pipe
    // }
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
                    println!("Got command {:?}!", command);
                    self.handle_command(command)
                }
                Err(_) => (),
            }

            let start = Instant::now();

            let mut image = image::ImageBuffer::new(self.img_width, self.img_height);
            self.mandelbrot(&mut image);

            // println!("Render took {}", start.elapsed().as_millis());

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

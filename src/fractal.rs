use image::{ImageBuffer, Rgb};
use num_complex::Complex;
use std::sync::mpsc::{sync_channel, Receiver};
use std::thread;
use std::time::Instant;
use std::{mem, sync::mpsc::Sender};
use std::{
    sync::{atomic::AtomicBool, mpsc::channel},
    time::Duration,
};
extern crate crossbeam;
extern crate num_cpus;
use crate::quadruple::{self, Quad};
use crossbeam::thread::scope;
use rayon::prelude::*;
use rug::{
    float::{self, FreeCache, Round},
    ops::{AddAssignRound, AssignRound, MulAssignRound},
    Float,
};
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Mutex};

#[cfg(all(
    target_arch = "x86_64",
    any(target_feature = "avx2", target_feature = "avx512f")
))]
use std::arch::x86_64::*;

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
            Command::LessIterations => self.limit -= if self.limit <= 200 { 0 } else { 200 },
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
                    self.limit = 200;
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
                    self.origin_x = -0.743643887037158704752191506114774;
                    self.origin_y = 0.131825904205311970493132056385139;
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

    pub fn mandelbrot_raw(&self, id: u32, height: u32, pixels: &mut [Rgb<u8>]) {
        let imgx = self.img_width;
        let imgy = self.img_height;

        for pixel_y in 0..height {
            for pixel_x in 0..self.img_width {
                let y_offset = pixel_y + id * height;
                let pinhole_center = self.pinhole_size / 2.0;
                let x0 = self.origin_x + (pixel_x as f64 / imgx as f64) * self.pinhole_size
                    - pinhole_center;
                let y0 = self.origin_y + (y_offset as f64 / imgy as f64) * self.pinhole_size
                    - pinhole_center;

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

                pixels[(pixel_y * self.img_height + pixel_x) as usize] =
                    color_rainbow(iteration, self.limit);
            }
        }
    }

    pub fn mandelbrot_quad(&self, id: u32, height: u32, pixels: &mut [Rgb<u8>]) {
        let imgx = self.img_width;
        let imgy = self.img_height;

        let pinhole_center = Quad::from(self.pinhole_size / 2.0);
        let pinhole_size = Quad::from(self.pinhole_size);

        let origin_x = Quad::from(self.origin_x);
        let origin_y = Quad::from(self.origin_y);

        for pixel_y in 0..height {
            let y_offset = pixel_y + id * height;

            for pixel_x in 0..self.img_width {
                // TODO: whole self struct has to keep quad value
                let x0 = origin_x + Quad::from(pixel_x as f64 / imgx as f64) * pinhole_size
                    - pinhole_center;
                let y0 = origin_y + Quad::from(y_offset as f64 / imgy as f64) * pinhole_size
                    - pinhole_center;

                let mut x = Quad::default();
                let mut y = Quad::default();
                let mut iteration = 0;

                let mut x2 = Quad::default();
                let mut y2 = Quad::default();
                let mut sum = Quad::default();

                while sum < Quad::from(4.0) && iteration < self.limit {
                    y = (x + x) * y + y0;

                    x = x2 - y2 + x0;

                    x2 = x * x;

                    y2 = y * y;

                    sum = x2 + y2;

                    iteration += 1;
                }

                pixels[(pixel_y * self.img_height + pixel_x) as usize] =
                    color_rainbow(iteration, self.limit);
            }
        }
    }

    pub fn mandelbrot_rug(&self, id: u32, height: u32, pixels: &mut [Rgb<u8>]) {
        const BIT_PRECISION: u32 = 64;
        let imgx = self.img_width;
        let imgy = self.img_height;

        let pinhole_center = Float::with_val(BIT_PRECISION, self.pinhole_size / 2.0);
        let pinhole_size = Float::with_val(BIT_PRECISION, self.pinhole_size);

        let origin_x = Float::with_val(BIT_PRECISION, self.origin_x);
        let origin_y = Float::with_val(BIT_PRECISION, self.origin_y);

        for pixel_y in 0..height {
            let y_offset = pixel_y + id * height;

            for pixel_x in 0..self.img_width {
                // TODO: whole self struct has to keep quad value
                let x0 = origin_x.clone()
                    + Float::with_val(BIT_PRECISION, pixel_x as f64 / imgx as f64)
                        * pinhole_size.clone()
                    - pinhole_center.clone();
                let y0 = origin_y.clone()
                    + Float::with_val(BIT_PRECISION, y_offset as f64 / imgy as f64)
                        * pinhole_size.clone()
                    - pinhole_center.clone();

                let mut x = Float::with_val(BIT_PRECISION, 0.0);
                let mut y = Float::with_val(BIT_PRECISION, 0.0);
                let mut iteration = 0;

                let mut x2 = Float::with_val(BIT_PRECISION, 0.0);
                let mut y2 = Float::with_val(BIT_PRECISION, 0.0);
                let mut sum = Float::with_val(BIT_PRECISION, 0.0);

                while sum < Float::with_val(BIT_PRECISION, 4.0) && iteration < self.limit {
                    y = (x.clone() + x.clone()) * y.clone() + y0.clone();

                    x = x2.clone() - y2.clone() + x0.clone();

                    x2 = x.clone() * x.clone();

                    y2 = y.clone() * y.clone();

                    sum = x2.clone() + y2.clone();

                    iteration += 1;
                }

                pixels[(pixel_y * self.img_height + pixel_x) as usize] =
                    color_rainbow(iteration, self.limit);
            }
        }
    }

    // TODO: cuda wrapper?
    // TODO: SIMD
    // https://www.officedaytime.com/simd512e/
    // https://nullprogram.com/blog/2015/07/10/
    // TODO: perturbation algo
    // http://math.ivanovo.ac.ru/dalgebra/Khashin/man2/Mandelbrot.pdf

    pub fn mandelbrot_simd_avx512(&self, id: u32, height: u32, pixels: &mut [Rgb<u8>]) {
        if !is_x86_feature_detected!("avx512f") {
            panic!("avx512f not supported on this platform :(");
        }

        let imgx = self.img_width;
        let imgy = self.img_height;

        let pinhole_center = self.pinhole_size / 2.0;
        let x0_offset = self.origin_x - pinhole_center;

        // SIMD part of code
        unsafe {
            // 4 doubles with bin representation of 0xff...
            let ff_mask = _mm512_cmp_pd_mask(_mm512_set1_pd(1.0), _mm512_set1_pd(4.0), _CMP_LE_OQ);

            for pixel_y in 0..height {
                let y_offset = pixel_y + id * height;
                let y0 = self.origin_y + (y_offset as f64 / imgy as f64) * self.pinhole_size
                    - pinhole_center;

                let y0 = _mm512_set1_pd(y0);

                // Step by 4, on every iteration we take 4 floats at once
                for pixel_x in (0..self.img_width).step_by(8) {
                    let mut iteration = [0, 0, 0, 0, 0, 0, 0, 0];

                    // let x0 = (pixel_x as f64 / imgx as f64) * self.pinhole_size + x0_offset;
                    // + x0_offset
                    let x0 = _mm512_add_pd(
                        // * self.pinhole_size
                        _mm512_mul_pd(
                            // pixel_x as f64 / imgx as f64
                            _mm512_div_pd(
                                _mm512_set_pd(
                                    (pixel_x + 7) as f64,
                                    (pixel_x + 6) as f64,
                                    (pixel_x + 5) as f64,
                                    (pixel_x + 4) as f64,
                                    (pixel_x + 3) as f64,
                                    (pixel_x + 2) as f64,
                                    (pixel_x + 1) as f64,
                                    pixel_x as f64,
                                ),
                                _mm512_set1_pd(imgx as f64),
                            ),
                            _mm512_set1_pd(self.pinhole_size),
                        ),
                        _mm512_set1_pd(x0_offset),
                    );

                    // let mut x = 0.0;
                    let mut x = _mm512_setzero_pd();
                    // let mut y = 0.0;
                    let mut y = _mm512_setzero_pd();
                    // let mut x2 = 0.0;
                    let mut x2 = _mm512_setzero_pd();
                    // let mut y2 = 0.0;
                    let mut y2 = _mm512_setzero_pd();
                    // let mut sum = 0.0;
                    let mut sum = _mm512_setzero_pd();

                    // TODO: try to change to range loop, should be no difference
                    let mut i = 0;
                    while i < self.limit {
                        i += 1;

                        // y = (x + x) * y + y0;
                        // + y0
                        y = _mm512_add_pd(
                            // * y
                            _mm512_mul_pd(
                                // x + x
                                _mm512_add_pd(x, x),
                                y,
                            ),
                            y0,
                        );

                        // x = x2 - y2 + x0;
                        // + x0
                        x = _mm512_add_pd(
                            // x2 - y2
                            _mm512_sub_pd(x2, y2),
                            x0,
                        );

                        // TODO: this is ABS for complex numbers, maybe there is some instrict?

                        // x2 = x * x;
                        x2 = _mm512_mul_pd(x, x);

                        // y2 = y * y;
                        y2 = _mm512_mul_pd(y, y);

                        // sum = x2 + y2;
                        sum = _mm512_add_pd(x2, y2);

                        // iteration += 1;
                        // TODO: other way to unpack?
                        let sum_unpacked: [f64; 8] = mem::transmute(sum);

                        // TODO: this can be SIMD too!
                        //iteration_test =
                        // _mm512_add_pd(_mm512_and_pd(mask, ff_mask), iteration_test);
                        // Returns NaN instead of 1.0 :\
                        for i in 0..sum_unpacked.len() {
                            iteration[i] = iteration[i] + (sum_unpacked[i] < 4.0) as u32;
                        }

                        // sum < 4.0, _CMP_LE_OQ == Less-than-or-equal (ordered, non-signaling)
                        let mask = _mm512_cmp_pd_mask(sum, _mm512_set1_pd(4.0), _CMP_LE_OQ);
                        // Mask will contain 0x1 per element if pred is true

                        // If mask is all 0, all points in the vector escaped 4.0 circle, break the loop
                        if mask == 0 {
                            break;
                        }
                    }

                    for i in 0..8 {
                        pixels[(pixel_y * self.img_height + pixel_x + i) as usize] =
                            color_rainbow(iteration[i as usize], self.limit);
                    }
                }
            } // unsafe
        }
    }

    pub fn mandelbrot_simd_avx2(&self, id: u32, height: u32, pixels: &mut [Rgb<u8>]) {
        if !is_x86_feature_detected!("avx2") {
            panic!("AVX2 not supported on this platform :(");
        }

        let imgx = self.img_width;
        let imgy = self.img_height;

        let pinhole_center = self.pinhole_size / 2.0;
        let x0_offset = self.origin_x - pinhole_center;

        // SIMD part of code
        unsafe {
            // 4 doubles with bin representation of 0xff...
            let ff_mask = _mm256_cmp_pd(_mm256_set1_pd(1.0), _mm256_set1_pd(4.0), _CMP_LE_OQ);

            let mut mask;

            for pixel_y in 0..height {
                let y_offset = pixel_y + id * height;
                let y0 = self.origin_y + (y_offset as f64 / imgy as f64) * self.pinhole_size
                    - pinhole_center;

                let y0 = _mm256_set1_pd(y0);

                // Step by 4, on every iteration we take 4 floats at once
                for pixel_x in (0..self.img_width).step_by(4) {
                    let mut iteration = [0, 0, 0, 0];

                    // let x0 = (pixel_x as f64 / imgx as f64) * self.pinhole_size + x0_offset;
                    // + x0_offset
                    let x0 = _mm256_add_pd(
                        // * self.pinhole_size
                        _mm256_mul_pd(
                            // pixel_x as f64 / imgx as f64
                            _mm256_div_pd(
                                _mm256_set_pd(
                                    (pixel_x + 3) as f64,
                                    (pixel_x + 2) as f64,
                                    (pixel_x + 1) as f64,
                                    pixel_x as f64,
                                ),
                                _mm256_set1_pd(imgx as f64),
                            ),
                            _mm256_set1_pd(self.pinhole_size),
                        ),
                        _mm256_set1_pd(x0_offset),
                    );

                    // let mut x = 0.0;
                    let mut x = _mm256_setzero_pd();
                    // let mut y = 0.0;
                    let mut y = _mm256_setzero_pd();
                    // let mut x2 = 0.0;
                    let mut x2 = _mm256_setzero_pd();
                    // let mut y2 = 0.0;
                    let mut y2 = _mm256_setzero_pd();
                    // let mut sum = 0.0;
                    let mut sum = _mm256_setzero_pd();

                    for i in 0..self.limit {
                        // y = (x + x) * y + y0;
                        // + y0
                        y = _mm256_add_pd(
                            // * y
                            _mm256_mul_pd(
                                // x + x
                                _mm256_add_pd(x, x),
                                y,
                            ),
                            y0,
                        );

                        // x = x2 - y2 + x0;
                        // + x0
                        x = _mm256_add_pd(
                            // x2 - y2
                            _mm256_sub_pd(x2, y2),
                            x0,
                        );

                        // TODO: this is ABS for complex numbers, maybe there is some instrict?

                        // x2 = x * x;
                        x2 = _mm256_mul_pd(x, x);

                        // y2 = y * y;
                        y2 = _mm256_mul_pd(y, y);

                        // sum = x2 + y2;
                        sum = _mm256_add_pd(x2, y2);

                        // iteration += 1;
                        // TODO: other way to unpack?
                        let sum_unpacked: [f64; 4] = mem::transmute(sum);

                        // TODO: this can be SIMD too!
                        //iteration_test =
                        // _mm256_add_pd(_mm256_and_pd(mask, ff_mask), iteration_test);
                        // Returns NaN instead of 1.0 :\
                        //
                        // __m256 mask = _mm256_cmp_ps(mag2, threshold, _CMP_LT_OS);
                        // mk = _mm256_add_ps(_mm256_and_ps(mask, one), mk);
                        //
                        for i in 0..sum_unpacked.len() {
                            iteration[i] = iteration[i] + (sum_unpacked[i] < 4.0) as u32;
                        }

                        // sum < 4.0, _CMP_LE_OQ == Less-than-or-equal (ordered, non-signaling)
                        mask = _mm256_cmp_pd(sum, _mm256_set1_pd(4.0), _CMP_LE_OQ);
                        // Mask will contain 0xfff... if pred is true, 0x000... otherwise

                        // If mask is all 0, all points in the vector escaped 4.0 circle, break the loop
                        if _mm256_testz_pd(mask, ff_mask) == 1 {
                            break;
                        }
                    }

                    for i in 0..4 {
                        pixels[(pixel_y * self.img_height + pixel_x + i) as usize] =
                            color_rainbow(iteration[i as usize], self.limit);
                    }
                }
            } // unsafe
        }
    }

    /// Use:
    /// Unsafe cell + waiting on threads - using atomic flags
    pub fn run_on_all_cpus_1(self) -> Pipe {
        let (img_send, img_rcv) = sync_channel(1);

        let (cmd_send, cmd_rcv) = channel();

        let pipe = Pipe {
            cmd_send: cmd_send,
            img_rcv: img_rcv,
        };

        thread::spawn(move || {
            let num_threads = num_cpus::get();

            let pixels_count = (self.img_width * self.img_height) as usize;

            let pixels: UnsafeCell<_> = vec![image::Rgb::from([0u8, 0, 0]); pixels_count].into();

            let chunk_size = pixels_count / num_threads;

            // // Share a context among threads, in order to do so,
            // // Make a clone of self (to avoid complaining that closures inside spawn outlives self)
            // // Wrap it in a mutex (to have safe access to context)
            // // Wrap into Arc (to have possibility to share it among threads - mutex does not have clone!)
            let mutex = Arc::new(Mutex::new(self.clone()));

            let mut ready = vec![];

            let mut threads = vec![];
            unsafe {
                for (id, chunk) in (*pixels.get()).chunks_mut(chunk_size).enumerate() {
                    let mutex = mutex.clone();
                    ready.push(Arc::new(AtomicBool::new(true)));
                    let ready = ready[id].clone();

                    threads.push(thread::spawn(move || {
                        let mut context;

                        loop {
                            // ready --> not consumed yet
                            while ready.load(Ordering::Acquire) {
                                thread::yield_now();
                            }

                            {
                                context = mutex.lock().unwrap().clone();
                            }

                            context.mandelbrot_raw(
                                id as u32,
                                context.img_height / num_threads as u32,
                                chunk,
                            );

                            ready.store(true, Ordering::Release);
                        }
                    }));
                }
            }

            loop {
                let start = Instant::now();
                match cmd_rcv.try_recv() {
                    Ok(command) => {
                        let mut context = mutex.lock().unwrap();
                        println!("Got command {:?}!", command);
                        context.handle_command(command);
                    }
                    Err(_) => (),
                }

                loop {
                    let finished = ready.iter().filter(|r| r.load(Ordering::Acquire)).count();

                    if finished == num_threads {
                        break;
                    }

                    thread::yield_now();
                }

                let image;
                unsafe {
                    image = image::ImageBuffer::from_fn(self.img_width, self.img_height, |x, y| {
                        (*pixels.get())[(y * self.img_width + x) as usize]
                    });
                }
                println!("render took {}", start.elapsed().as_millis());

                img_send.send(image).unwrap();

                {
                    let mut context = mutex.lock().unwrap();
                    context.pinhole_size *= context.pinhole_step;
                }

                for r in &ready {
                    r.store(false, Ordering::Release);
                }
            }
        });

        pipe
    }

    /// Use rayon's par iterator
    pub fn run_on_rayon(mut self) -> Pipe {
        let (img_send, img_rcv) = sync_channel(1);

        let (cmd_send, cmd_rcv) = channel();

        let pipe = Pipe {
            cmd_send: cmd_send,
            img_rcv: img_rcv,
        };

        thread::spawn(move || {
            let num_threads = num_cpus::get();

            let pixels_count = (self.img_width * self.img_height) as usize;

            let mut pixels = vec![image::Rgb::from([0u8, 0, 0]); pixels_count];

            let chunk_size = pixels_count / num_threads;

            loop {
                let start = Instant::now();
                match cmd_rcv.try_recv() {
                    Ok(command) => {
                        println!("Got command {:?}!", command);
                        self.handle_command(command);
                    }
                    Err(_) => (),
                }

                let _: Vec<_> = pixels
                    .par_chunks_mut(chunk_size)
                    .enumerate()
                    .map(|(id, chunk)| {
                        // self.mandelbrot_quad(
                        //     id as u32,
                        //     self.img_height / num_threads as u32,
                        //     chunk,
                        // );

                        self.mandelbrot_rug(id as u32, self.img_height / num_threads as u32, chunk);

                        // self.mandelbrot_simd_avx2(id as u32, self.img_height / num_threads as u32, chunk)
                    })
                    .collect();

                let image = image::ImageBuffer::from_fn(self.img_width, self.img_height, |x, y| {
                    pixels[(y * self.img_width + x) as usize]
                });

                println!("render took {}", start.elapsed().as_millis());

                img_send.send(image).unwrap();

                self.pinhole_size *= self.pinhole_step;
            }
        });

        pipe
    }

    pub fn run_on_rayon_simd(mut self) -> Pipe {
        let (img_send, img_rcv) = sync_channel(1);

        let (cmd_send, cmd_rcv) = channel();

        let pipe = Pipe {
            cmd_send: cmd_send,
            img_rcv: img_rcv,
        };

        thread::spawn(move || {
            let num_threads = num_cpus::get();

            let pixels_count = (self.img_width * self.img_height) as usize;

            let mut pixels = vec![image::Rgb::from([0u8, 0, 0]); pixels_count];

            let chunk_size = pixels_count / num_threads;

            loop {
                let start = Instant::now();
                match cmd_rcv.try_recv() {
                    Ok(command) => {
                        println!("Got command {:?}!", command);
                        self.handle_command(command);
                    }
                    Err(_) => (),
                }

                let _: Vec<_> = pixels
                    .par_chunks_mut(chunk_size)
                    .enumerate()
                    .map(|(id, chunk)| {
                        self.mandelbrot_simd_avx2(
                            id as u32,
                            self.img_height / num_threads as u32,
                            chunk,
                        );
                    })
                    .collect();

                let image = image::ImageBuffer::from_fn(self.img_width, self.img_height, |x, y| {
                    pixels[(y * self.img_width + x) as usize]
                });

                println!("render took {}", start.elapsed().as_millis());

                img_send.send(image).unwrap();

                self.pinhole_size *= self.pinhole_step;
            }
        });

        pipe
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
                    println!("Got command {:?}!", command);
                    self.handle_command(command)
                }
                Err(_) => (),
            }

            let start = Instant::now();

            let mut image = image::ImageBuffer::new(self.img_width, self.img_height);
            self.mandelbrot(&mut image);

            println!("Render took {}", start.elapsed().as_millis());

            img_send.send(image).unwrap();

            self.pinhole_size *= self.pinhole_step;
        });

        pipe
    }

    pub fn run_on_thread_simd(mut self) -> Pipe {
        let (img_send, img_rcv) = channel();

        let (cmd_send, cmd_rcv) = channel();

        let pipe = Pipe {
            cmd_send: cmd_send,
            img_rcv: img_rcv,
        };

        thread::spawn(move || loop {
            let pixels_count = (self.img_width * self.img_height) as usize;
            let mut pixels = vec![image::Rgb::from([0u8, 0, 0]); pixels_count];

            match cmd_rcv.try_recv() {
                Ok(command) => {
                    println!("Got command {:?}!", command);
                    self.handle_command(command)
                }
                Err(_) => (),
            }

            let start = Instant::now();

            self.mandelbrot_simd_avx2(0, self.img_height, &mut pixels);

            let image = image::ImageBuffer::from_fn(self.img_width, self.img_height, |x, y| {
                pixels[(y * self.img_width + x) as usize]
            });

            println!("Render took {}", start.elapsed().as_millis());

            img_send.send(image).unwrap();

            self.pinhole_size *= self.pinhole_step;
        });

        pipe
    }
}

fn _color_gray(iteration: u32, limit: u32) -> image::Rgb<u8> {
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

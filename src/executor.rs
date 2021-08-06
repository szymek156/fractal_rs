use std::{
    sync::mpsc::{channel, sync_channel, Receiver, Sender},
    thread,
    time::Instant,
};

use image::{ImageBuffer, Rgb};
use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

// Thanks to exact picks, there are no circular references!!
use crate::{fractal_builder::Context, fractals::Floating};

pub type OutBuffer = ImageBuffer<Rgb<u8>, Vec<u8>>;
#[derive(Debug)]

pub enum FineDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub enum Command {
    ZoomOut,
    ZoomIn,
    LessIterations,
    MoreIterations,
    FineTune(FineDirection),
    // f64 is enough
    ChangeOrigin(f64, f64),
    SetPOI(u32),
    GetState,
}

pub struct Pipe {
    pub img_rcv: Receiver<OutBuffer>,
    pub cmd_send: Sender<Command>,
}

pub enum ExecutorKind {
    SingleThread,
    Rayon,
}

/// Launches threaded backend for fractal computation.
/// Returns a Pipe:
/// img_rcv - for getting ready images of fractal
/// cmd_send - for sending commands to the Executor, like change of PoI, etc.
pub trait Executor<F: Floating> {
    fn execute(self, context: Context<F>) -> Pipe;


    fn handle_command(&self, command: Command, context: &mut Context<F>) {
        match command {
            Command::ZoomOut => context.pinhole_step += F::from(0.1),
            Command::ZoomIn => context.pinhole_step -= F::from(0.1),
            Command::LessIterations => {
                context.poi.limit -= if context.poi.limit <= 200 { 0 } else { 200 }
            }
            Command::MoreIterations => context.poi.limit += 200,
            Command::FineTune(dir) => {
                let tune = context.poi.pinhole_size * F::from(0.15);
                match dir {
                    FineDirection::Up => context.poi.origin_y += tune,
                    FineDirection::Down => context.poi.origin_y -= tune,
                    FineDirection::Left => context.poi.origin_x -= tune,
                    FineDirection::Right => context.poi.origin_x += tune,
                }
            }
            Command::ChangeOrigin(x, y) => {
                let pinhole_center = context.poi.pinhole_size / F::from(2.0);

                context.poi.origin_x = context.poi.origin_x
                    + ((F::from(x) / F::from(context.img_width as f64)) * context.poi.pinhole_size)
                    - pinhole_center;

                // * -1.0 because Y values increase in down direction
                context.poi.origin_y = context.poi.origin_y
                    + (((F::from(y) / F::from(context.img_height as f64))
                        * context.poi.pinhole_size)
                        - pinhole_center)
                        * F::from(-1.0);
            }
            Command::SetPOI(poi) => match poi {
                0 => {
                    context.poi.origin_x = F::from(0.0);
                    context.poi.origin_y = F::from(0.0);
                    context.poi.pinhole_size = F::from(4.0);
                    context.pinhole_step = F::from(1.0);
                    context.poi.limit = 200;
                }
                1 => {
                    context.poi.origin_x = F::from(-1.2583384664947936);
                    context.poi.origin_y = F::from(-0.032317669198187016);
                }
                2 => {
                    context.poi.origin_x = F::from(-1.2487780999747029);
                    context.poi.origin_y = F::from(0.071802096973029209);
                }
                3 => {
                    context.poi.origin_x = F::from(-1.2583385189936513);
                    context.poi.origin_y = F::from(-0.032317635405726151);
                }
                4 => {
                    context.poi.origin_x = F::from(-1.2583384664947908);
                    context.poi.origin_y = F::from(-0.032317669198180785);
                }
                5 => {
                    context.poi.origin_x = F::from(-1.4780998580724920);
                    context.poi.origin_y = F::from(-0.0029962325962097328);
                }
                6 => {
                    context.poi.origin_x = F::from(-0.743643887037158704752191506114774);
                    context.poi.origin_y = F::from(0.131825904205311970493132056385139);
                }
                7 => {
                    context.poi.origin_x = F::from(-1.768611136076306);
                    context.poi.origin_y = F::from(-0.001266863985331);
                }
                8 => {
                    context.poi.origin_x = F::from(-1.7686112281079116);
                    context.poi.origin_y = F::from(-0.0012668963162883458);
                }
                9 => {
                    // self.origin_x = -1.2568840461035797;
                    // self.origin_y = 0.3796264149862358;

                    context.poi.origin_x = F::from(-1.6291627176190138);
                    context.poi.origin_y = F::from(-0.020224379647719847);
                }

                _ => (),
            },
            Command::GetState => {
                println!("Current position: {:#?}", context.poi);
                println!("Zoom: {:#?}", F::from(4.0) / context.poi.pinhole_size);
            }
        }
    }
}

pub struct Rayon;

impl<F: Floating> Executor<F> for Rayon {
    fn execute(self, context: Context<F>) -> Pipe {
        let (img_send, img_rcv) = sync_channel(1);

        let (cmd_send, cmd_rcv) = channel();

        let pipe = Pipe {
            cmd_send: cmd_send,
            img_rcv: img_rcv,
        };

        thread::spawn(move || {
            // Changing mutability here
            // TODO: is it better way to do it???
            let mut context = context;
            let num_threads = num_cpus::get();

            let pixels_count = (context.img_width * context.img_height) as usize;

            let mut pixels = vec![image::Rgb::from([0u8, 0, 0]); pixels_count];

            let chunk_size = pixels_count / num_threads;

            loop {
                let start = Instant::now();
                match cmd_rcv.try_recv() {
                    Ok(command) => {
                        println!("Got command {:?}!", command);
                        self.handle_command(command, &mut context);
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
                    })
                    .collect();

                let image =
                    image::ImageBuffer::from_fn(context.img_width, context.img_height, |x, y| {
                        pixels[(y * context.img_width + x) as usize]
                    });

                println!("render took {}", start.elapsed().as_millis());

                img_send.send(image).unwrap();

                context.poi.pinhole_size *= context.pinhole_step;
            }
        });

        pipe
    }
}

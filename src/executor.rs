use std::{sync::mpsc::{Receiver, Sender, channel, sync_channel}, thread, time::Instant};

use image::{ImageBuffer, Rgb};


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
    ChangeOrigin(f64, f64),
    SetPOI(u32),
    GetState,
}

pub struct Pipe {
    pub img_rcv: Receiver<OutBuffer>,
    pub cmd_send: Sender<Command>,
}


pub enum ExecutorType {
    SingleThread,
    Rayon
}

/// Launches threaded backend for fractal computation.
/// Returns a Pipe:
/// img_rcv - for getting ready images of fractal
/// cmd_send - for sending commands to the Executor, like change of PoI, etc.
pub trait Executor {
    fn execute(self) -> Pipe;
}

pub struct Rayon;

impl Executor for Rayon {
    fn execute(self) -> Pipe {
        let (img_send, img_rcv) = sync_channel(1);

        let (cmd_send, cmd_rcv) = channel();

        let pipe = Pipe {
            cmd_send: cmd_send,
            img_rcv: img_rcv,
        };

        // thread::spawn(move || {
        //     let num_threads = num_cpus::get();

        //     let pixels_count = (self.img_width * self.img_height) as usize;

        //     let mut pixels = vec![image::Rgb::from([0u8, 0, 0]); pixels_count];

        //     let chunk_size = pixels_count / num_threads;

        //     loop {
        //         let start = Instant::now();
        //         match cmd_rcv.try_recv() {
        //             Ok(command) => {
        //                 println!("Got command {:?}!", command);
        //                 self.handle_command(command);
        //             }
        //             Err(_) => (),
        //         }

        //         let _: Vec<_> = pixels
        //             .par_chunks_mut(chunk_size)
        //             .enumerate()
        //             .map(|(id, chunk)| {
        //                 self.mandelbrot_quad(
        //                     id as u32,
        //                     self.img_height / num_threads as u32,
        //                     chunk,
        //                 );
        //             })
        //             .collect();

        //         let image = image::ImageBuffer::from_fn(self.img_width, self.img_height, |x, y| {
        //             pixels[(y * self.img_width + x) as usize]
        //         });

        //         println!("render took {}", start.elapsed().as_millis());

        //         img_send.send(image).unwrap();

        //         self.pinhole_size *= self.pinhole_step;

        //     }
        // });

        pipe
    }
}

use image::{ImageBuffer, Rgb};
use std::sync::mpsc::{Receiver, Sender};

use crate::executor::Command;
pub struct Pipe {
    pub img_rcv: Receiver<OutBuffer>,
    pub cmd_send: Sender<Command>,
}

pub type OutBuffer = ImageBuffer<Rgb<u8>, Vec<u8>>;